use crate::clangd::ClangdUtility;
use crate::symbols::{SymbolId, SymbolKind};

use griff::ChunkStream;

#[derive(Debug, Clone, Default)]
#[repr(u8)]
pub enum RelationKind {
    #[default]
    BaseOf,
    OverriddenBy,
}
impl From<u8> for RelationKind {
    fn from(b: u8) -> Self {
        use RelationKind::*;
        match b {
            1 => OverriddenBy,
            _ => BaseOf
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rela {
    pub subject: SymbolId,
    pub predicate: RelationKind,
    pub object: SymbolId,
}
impl ClangdUtility for Rela{}

impl Rela {
    pub fn parse(buf: &ChunkStream) -> Vec<Rela> {
        let mut rela: Vec<Rela> = vec![];
        let mut cursor: usize = 0;
        let _data = buf.data.clone();
        let data = _data.as_slice();
        if data.len() == 0 {
            return rela;
        }

        loop {
            let mut r: Rela = Default::default();
            r.subject = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
            cursor += 8;
            r.predicate = RelationKind::from(data[cursor]);
            cursor += 1;
            r.object = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
            cursor += 8;

            rela.push(r);
            if cursor >= data.len() {
                break;
            }
        }

        rela
    }
}