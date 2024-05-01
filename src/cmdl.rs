use griff::ChunkStream;
use crate::clangd::ClangdUtility;

#[derive(Debug, Clone, Default)]
pub struct Cmdl {
    pub directory: String,
    pub cmdl: Vec<String>,
}
impl ClangdUtility for Cmdl{}

impl Cmdl {
    #[allow(dead_code)]
    pub fn parse(stream: &ChunkStream, string_tables: &Vec<String>) -> Vec<Self> {
        let mut cmdlines: Vec<Cmdl> = vec![];
        let _data = stream.data.clone();
        let data: &[u8] = _data.as_slice();
        if data.len() == 0 {
            return cmdlines;
        }
        let mut cursor: usize = 0;
        let mut idx: u32;
        while cursor < data.len() {
            let mut c: Cmdl = Default::default();
            let (sz, content) = Self::get_string(data.get(cursor..).unwrap(), string_tables);
            c.directory = content;
            cursor += sz;
            let (sz, len) = Self::get_varint(data.get(cursor..).unwrap());
            cursor += sz;
            idx = 0;
            while idx < len {
                let (sz, content) = Self::get_string(data.get(cursor..).unwrap(), string_tables);
                c.cmdl.push(content);
                cursor += sz;
                idx += 1;
            }
            cmdlines.push(c);
        }

        cmdlines
    }
}
