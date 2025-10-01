#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use cli_lib::js_executor::JsExecutor;
use compiler_lib::{CompilationResult, State};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "PolySubML Compiler CLI")]
struct Args {
    /// ML files to compile
    files: Vec<PathBuf>,

    /// Directory to cache JS execution results (optional)
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut state = State::new();
    let js_executor = JsExecutor::new(args.cache_dir);

    for fname in args.files {
        println!("Processing {}", fname.display());
        let data = fs::read_to_string(&fname).unwrap();

        let t0 = Instant::now();
        let res = state.process(&data);
        dbg!(t0.elapsed());

        println!("{}", res);

        if let CompilationResult::Success(js_code) = res {
            println!("\nExecuting...");
            match js_executor.execute_js(&js_code) {
                Ok(output) => {
                    println!("Output:\n{}", output);
                }
                Err(e) => {
                    eprintln!("Execution error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use compiler_lib::CompilationResult;

    #[test]
    fn run_tests() {
        let mut state = State::new();

        let data = fs::read_to_string("../tests/combined.ml").unwrap();
        for part in data.split("###") {
            let s = part.trim();
            if s.is_empty() {
                continue;
            }

            let (kind, s) = s.split_once('\n').unwrap();
            // println!("{} {}", kind, s);

            match state.process(s) {
                CompilationResult::Success(s) => {
                    if kind != "Good" {
                        println!("Unexpectedly passed:\n{}", s);
                        assert_eq!(kind, "Good");
                    }
                }
                CompilationResult::Error(e) => {
                    if kind != "Bad" {
                        println!("Unexpected error:\n{}", e);
                        assert_eq!(kind, "Bad");
                    }
                }
            }
        }
    }
}
