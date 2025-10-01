#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;
use std::fs;
use std::time::Instant;

use compiler_lib::State;

fn main() {
    let mut state = State::new();
    for fname in env::args().skip(1) {
        println!("Processing {}", fname);
        let data = fs::read_to_string(fname).unwrap();
        // println!(">> {}", data);

        let t0 = Instant::now();
        let res = state.process(&data);
        dbg!(t0.elapsed());

        println!("{}", res);
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
