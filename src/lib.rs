//! clangd-parser
//! Parse the clangd output to leverage in other tools, such as test generation.

pub mod clangd;
pub mod symbols;
pub mod rela;
pub mod refs;
pub mod srcs;
pub mod cmdl;

use async_std::task;

use std::fs;
use std::path::PathBuf;
use std::collections::BTreeMap;
use crate::symbols::SymbolKind;

/// Given a root directory containing .cache/index, parse the IDX files
pub fn run(p: &PathBuf) -> clangd::ClangdDatabase {
    let mut db = task::block_on(_run(p));
    #[cfg(feature="post-process")]
    post_process(&mut db);
    db
}

#[cfg(feature="post-process")]
fn post_process(db: &mut clangd::ClangdDatabase) {
    // for each name
    for entry in db.name.iter() {
        let sym = entry.1;

        match sym.syminfo.kind {
            // For variables, check if declared in an H file.
            // If so, map id in corresponding file entry
            SymbolKind::Variable => {
                let decl_file = sym.canonical_declaration.file_uri.rsplit_once("/");
                if decl_file.is_none() {
                    debug_assert!(false);
                }
                let decl_file = decl_file.unwrap().1;
                if decl_file.ends_with(".h") {
                    let hfile = db.file.get_mut(&decl_file.to_string());
                    if hfile.is_none() {
                        debug_assert!(false);
                    }
                    let hfile = hfile.unwrap();
                    hfile.variable_declarations.push(sym.id.clone());
                }
            },
            _ => ()
        }
    }
}

async fn _run(p: &PathBuf) -> clangd::ClangdDatabase {
    let mut to_file: clangd::ClangdFileMap = BTreeMap::new();
    let mut to_id: clangd::ClangdIdMap = BTreeMap::new();
    let mut to_name: clangd::ClangdNameMap = BTreeMap::new();
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
                let fname = e.file_name().to_str().unwrap().to_string();
                let parts: Vec<&str> = fname.split(".").collect();
                let fname = format!("{}.{}", parts[0], parts[1]);
                let db = _ret.unwrap();
                if !to_file.contains_key(&fname) {
                    let _ = to_file.insert(fname, db.clone());
                }
                for sym in db.symbols.data.iter() {
                    if !to_id.contains_key(&sym.id) {
                        let _ = to_id.insert(sym.id.clone(), sym.clone());
                    }
                    if !to_name.contains_key(&sym.name) {
                        let _ = to_name.insert(sym.name.clone(), sym.clone());
                    }
                }
            }
        }
    }

    clangd::ClangdDatabase{ file: to_file, id: to_id, name: to_name }
}
