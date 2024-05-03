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
    let mut to_name: clangd::ClangdNameMap = HashMap::new();
    let mut path = p.join(".cache");
    if !path.exists() {
        panic!("Unable to find .cache!");
    }
    path = path.join("clangd").join("index");
    if !path.exists() {
        panic!("Has clangd been run?");
    }

    let rd = fs::read_dir(path.as_path()).unwrap();
    for entry in rd {
        let e = entry.unwrap();
        if e.file_type().unwrap().is_file() {
            let _ret = clangd::ClangdFile::parse(e.path()).await;
            if _ret.is_ok() {
                let mut fname = e.file_name().to_str().unwrap().to_string();
                let parts: Vec<&str> = fname.split('.').collect();
                fname = format!("{}.{}", parts[0], parts[1]);
                let db = _ret.unwrap();
                let _ = to_file.entry(fname).or_insert(db.clone());
                for sym in db.symbols.data.iter() {
                    let _ = to_id.entry(sym.id.clone()).or_insert(sym.clone());
                    let _ = to_name.entry(sym.name.clone()).or_insert(sym.clone());
                }
            }
        }
    }

    clangd::ClangdDatabase{ file: to_file, id: to_id, name: to_name }
}
