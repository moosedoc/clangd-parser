pub mod clangd;
pub mod symbols;
pub mod rela;
pub mod refs;
pub mod srcs;
pub mod cmdl;

use async_std::task;

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

pub fn run(p: PathBuf) -> clangd::ClangdDatabase {
    task::block_on(_run(p))
}

async fn _run(p: PathBuf) -> clangd::ClangdDatabase {
    let mut to_file: clangd::ClangdFileMap = HashMap::new();
    let mut to_id: clangd::ClangdIdMap = HashMap::new();
    let mut p = p.join(".cache");
    if !p.exists() {
        panic!("Unable to find .cache!");
    }
    p = p.join("clangd").join("index");
    if !p.exists() {
        panic!("Has clangd been run?");
    }

    let rd = fs::read_dir(p.as_path()).unwrap();
    for entry in rd {
        let e = entry.unwrap();
        if e.file_type().unwrap().is_file() {
            let _ret = clangd::ClangdFile::parse(e.path()).await;
            if _ret.is_ok() {
                let db = _ret.unwrap();
                let mut fname = e.file_name().to_str().unwrap().to_string();
                let parts: Vec<&str> = fname.split('.').collect();
                fname = format!("{}.{}", parts[0], parts[1]);
                let _ = to_file.entry(fname).or_insert(db.clone());
                for sym in db.symbols.data.iter() {
                    let _ = to_id.entry(sym.id).or_insert(sym.clone());
                }
            }
        }
    }

    clangd::ClangdDatabase{ file: to_file, id: to_id }
}
