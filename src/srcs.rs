use crate::clangd::ClangdUtility;

use griff::ChunkStream;

#[derive(Debug, Clone, Default)]
#[repr(u8)]
pub enum SourceFlags {
    #[default]
    None,

    IsTU = 1 << 0,
    HadErrors = 1 << 1,
}
impl From<u8> for SourceFlags {
    fn from(b: u8) -> Self {
        use SourceFlags::*;
        match b {
            1 => IsTU,
            2 => HadErrors,
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Srcs {
    pub flags: SourceFlags,
    pub uri: String,
    pub digest: [u8; 8],
    pub direct_includes: Vec<String>,

}
impl ClangdUtility for Srcs{}

impl Srcs {
    #[allow(dead_code)]
    pub fn parse(stream: &ChunkStream, string_table: &Vec<String>) -> Vec<Srcs> {
        let mut cursor: usize = 0;
        let mut idx: u32;
        let _data = stream.data.clone();
        let data = _data.as_slice();
        let mut srcs: Vec<Srcs> = vec![];
        if data.len() == 0 {
            return srcs;
        }

        loop {
            let mut src: Srcs = Default::default();
            src.flags = SourceFlags::from(data[cursor]);
            cursor += 1;
            let (sz, content) = Self::get_string(data.get(cursor..).unwrap(), string_table);
            src.uri = content;
            cursor += sz;
            src.digest = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
            cursor += 8;
            let (sz, len) = Self::get_varint(data.get(cursor..).unwrap());
            cursor += sz;
            idx = 0;
            while idx < len {
                let (sz, content) = Self::get_string(data.get(cursor..).unwrap(), string_table);
                src.direct_includes.push(content);
                cursor += sz;
                idx += 1;
            }
            srcs.push(src);
            
            if cursor >= data.len() {
                break;
            }
        }
        srcs
    }
}