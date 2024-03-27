mod clangd;
mod rela;
mod refs;
mod srcs;
mod symbols;
mod cmdl;

use clap::Parser;

use std::path::PathBuf;
use std::time::SystemTime;

use clangd_parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Path to repo root
    #[arg(short='d', long, default_value_t=String::from("."))]
    path: String,
}

fn main() {
    let timer = SystemTime::now();
    let args = Cli::parse();
    let p = PathBuf::from(args.path.as_str());

    let results = clangd_parser::run(p);
    println!("Execution took {:.2}s.", timer.elapsed().unwrap().as_secs_f32());
}