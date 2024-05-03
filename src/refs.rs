use crate::symbols::{SymbolId, SymbolLocation, SymbolKind};
use crate::clangd::ClangdUtility;

use griff::ChunkStream;

#[derive(Debug, Clone, Default)]
pub struct RefReferences {
    pub kind: SymbolKind,
    pub location: SymbolLocation,
    pub container_id: SymbolId,
}

#[derive(Debug, Clone, Default)]
pub struct Refs {
    pub id: SymbolId,
    pub cnt: usize,
    pub refs: Vec<RefReferences>,
}
impl ClangdUtility for Refs{}

impl Refs {
    pub fn parse(buf: &ChunkStream, string_table: &Vec<String>) -> Vec<Refs> {
        let mut refs: Vec<Refs> = vec![];
        let mut cursor: usize = 0;
        let _data = buf.data.clone();
        let data = _data.as_slice();
        if data.len() == 0 {
            return refs;
        }

        loop {
            let mut r: Refs = Default::default();
            r.id = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
            cursor += 8;
            let (sz, content) = Self::get_varint(&data[cursor..]);
            cursor += sz;
            for _ in 0..content {
                let mut rr: RefReferences = Default::default();
                rr.kind = SymbolKind::from(data[cursor]);
                cursor += 1;
                let (sz, loc) = SymbolLocation::get_location(&data.get(cursor..).unwrap(), string_table);
                rr.location = loc;
                cursor += sz;
                rr.container_id = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
                cursor += 8;

                r.refs.push(rr);
            }

            refs.push(r);
            if cursor >= data.len() {
                break;
            }
        }
        refs
    }
}
