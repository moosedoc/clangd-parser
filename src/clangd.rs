use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use griff::*;

use crate::symbols;
use crate::rela;
use crate::refs;
use crate::srcs;
use crate::cmdl;

#[derive(Debug)]
pub enum ParseError {
    CannotReadFile,
    RiffError(ChunkError),
}
type ParseReturn = Result<ClangdFile, ParseError>;

pub type ClangdFileMap = HashMap<String, ClangdFile>;
pub type ClangdIdMap = HashMap<symbols::SymbolId, symbols::Symbol>;
#[derive(Debug, Clone)]
pub struct ClangdDatabase {
    pub file: ClangdFileMap,
    pub id: ClangdIdMap,
}

pub trait ClangdUtility {
    fn get_varint(buf: &[u8]) -> (usize, u32) {
        let mut bytes_read: usize = 0;
        let mut varint: u32 = 0;
        let mut shift: u32 = 0;
        
        loop {
            let b = buf[bytes_read];
            let cont = (b >> 7) & 1;
            let tmp = (0x7F & b) as u32;
            varint |= tmp << shift;
            bytes_read += 1;
            shift += 7;
            if cont != 1
            || !(shift < 32) {
                break;
            }
        }
        (bytes_read, varint)
    }

    fn get_string(buf: &[u8], string_table: &Vec<String>) -> (usize, String) {
        let bytes_read: usize;
        let mut s: String = String::new();
        let (sz, idx) = Self::get_varint(buf);
        bytes_read = sz;
        if (idx as usize) < string_table.len() {
            s = string_table[ idx as usize ].clone();
        }
        s = s.trim_end_matches("\0").to_string();
        (bytes_read, s)
    }

    fn get_u32(buf: &[u8]) -> (usize, u32) {
        let mut ret: u32 = 0;
        for i in 0..4 {
            ret |= (buf[i] as u32) << i*8;
        }
        (4, ret)
    }

    fn decompress(buf: &[u8]) -> Vec<u8> {
        use std::io::{BufReader, Read};
        use libflate::zlib::Decoder;

        let reader = BufReader::new(buf);
        let mut v: Vec<u8> = vec![];
        let mut decoder = Decoder::new(reader).unwrap();
        decoder.read_to_end(&mut v).unwrap();
        v
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClangdSymbols {
    pub data: Vec<symbols::Symbol>,
}

#[derive(Debug, Clone, Default)]
pub struct ClangdRelations {
    pub data: Vec<rela::Rela>,
}

#[derive(Debug, Clone, Default)]
pub struct ClangdCmdLine {
    pub data: Vec<cmdl::Cmdl>,
}

#[derive(Debug, Clone, Default)]
pub struct ClangdSources {
    pub data: Vec<srcs::Srcs>,
}

#[derive(Debug, Clone, Default)]
pub struct ClangdMetaData {
    version: [u8;4],
}

#[derive(Debug, Clone, Default)]
pub struct ClangdReferences {
    pub data: Vec<refs::Refs>,
}

#[derive(Debug, Clone, Default)]
pub struct ClangdFileType {
    ftype: [u8;4],
}

#[derive(Debug, Clone, Default)]
pub struct ClangdFile {
    // string table from which everything references
    pub string: Vec<String>,
    // symbols defined in the file
    pub symbols: ClangdSymbols,
    // ?
    pub relations: ClangdRelations,
    // cmdl args to compile file
    pub cmdline: ClangdCmdLine,
    // ?
    pub sources: ClangdSources,
    // clangd idx metadata
    pub meta: ClangdMetaData,
    // all references in the file, internal & external
    pub references: ClangdReferences,
    // RIFF file type
    pub file_type: ClangdFileType,
}

impl ClangdFile {
    pub async fn parse(p: PathBuf) -> ParseReturn {
        let _contents = fs::read(p.as_path());
        if _contents.is_err() {
            return Err(ParseError::CannotReadFile);
        }
    
        let contents = _contents.unwrap();
        let data = contents.as_slice();
        let riff = Riff::parse(data).await;
        if riff.is_err() {
            return Err(ParseError::RiffError(riff.unwrap_err()));
        }
        let cd: ClangdFile = ClangdFile::consume_riff(&riff.unwrap());
        Ok(cd)
    }
    
    fn consume_riff(riff: &Riff) -> ClangdFile {
        let mut cd: ClangdFile = Default::default();
        if let Some(r) = &riff.chunk {
            match r.id {
                ChunkId::Riff => {
                    match &r.data {
                        ChunkData::RiffData(x) => {
                            if x.file_type == *CDIX {
                                for child in &x.data {
                                    match child.id {
                                        ChunkId::Stri => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.string = ClangdFile::consume_string(&data);
                                        },
                                        ChunkId::Symb => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.symbols = ClangdFile::consume_symbols(&data, &cd.string);
                                        },
                                        ChunkId::Srcs => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.sources = ClangdFile::consume_sources(&data, &cd.string);
                                        },
                                        ChunkId::Rela => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.relations = ClangdFile::consume_relations(&data);
                                        },
                                        ChunkId::Refs => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.references = ClangdFile::consume_references(&data, &cd.string);
                                        },
                                        ChunkId::Cmdl => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.cmdline = ClangdFile::consume_cmdline(&data, &cd.string);
                                        },
                                        ChunkId::Meta => {
                                            let data: ChunkStream = ClangdFile::get_stream(&child.data);
                                            cd.meta = ClangdFile::consume_metadata(&data);
                                        },                                        
                                        _ => todo!()
                                    }
                                }
                            }
                        },
                        _ => todo!()
                    }
                },
                _ => todo!()
            }
        }
    
        cd
    }

    fn consume_string(data: &ChunkStream) -> Vec<String> {
        let v: Vec<String>;
        let stream = data.data.clone();
        let buf: &[u8] = stream.as_slice();
        let (sz, compr_sz) = Self::get_u32(buf);
        if compr_sz == 0 {
            // uncompressed
            v = Self::get_strings(buf.get(sz..).unwrap());
        }
        else {
            // compressed
            let decomp = Self::decompress(buf.get(sz..).unwrap());
            let buf = decomp.as_slice();
            v = Self::get_strings(buf);
        }
        v
    }

    fn get_strings(buf: &[u8]) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        let mut s: String = String::new();
        for i in 0..buf.len() {
            s.push(buf[i] as char);
            if buf[i] == b'\0' {
                v.push(s.clone());
                s.clear();
            }
        }
        v
    }

    fn consume_symbols(data: &ChunkStream, string_table: &Vec<String>) -> ClangdSymbols {
        let mut cs: ClangdSymbols = Default::default();
        cs.data = symbols::Symbol::parse(data, string_table);
        cs
    }

    fn consume_sources(data: &ChunkStream, string_table: &Vec<String>) -> ClangdSources {
        let mut cs: ClangdSources = Default::default();
        cs.data = srcs::Srcs::parse(data, string_table);
        cs
    }

    fn consume_relations(data: &ChunkStream) -> ClangdRelations {
        let mut cr: ClangdRelations = Default::default();
        cr.data = rela::Rela::parse(data);
        cr
    }

    fn consume_references(data: &ChunkStream, string_table: &Vec<String>) -> ClangdReferences {
        let mut cr: ClangdReferences = Default::default();
        cr.data = refs::Refs::parse(data, string_table);
        cr
    }

    fn consume_cmdline(data: &ChunkStream, string_table: &Vec<String>) -> ClangdCmdLine {
        let mut ccl: ClangdCmdLine = Default::default();
        ccl.data = cmdl::Cmdl::parse(data, string_table);
        ccl
    }

    fn consume_metadata(data: &ChunkStream) -> ClangdMetaData {
        let mut cm: ClangdMetaData = Default::default();
        cm.version = data.data.clone().as_slice().try_into().unwrap();
        cm
    }

    fn get_stream(cd: &ChunkData) -> ChunkStream {
        match cd {
            ChunkData::StreamData(x) => x.clone(),
            _ => todo!()
        }
    }
}
impl ClangdUtility for ClangdFile{}
