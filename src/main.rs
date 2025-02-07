#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;
use std::fs;
use std::time::Instant;

use polysubml_demo::State;

fn main() {
    let mut state = State::new();
    for fname in env::args().skip(1) {
        println!("Processing {}", fname);
        let data = fs::read_to_string(fname).unwrap();
        // println!(">> {}", data);

        let t0 = Instant::now();
        let res = if state.process(&data) {
            state.get_output().unwrap()
        } else {
            state.get_err().unwrap()
        };
        dbg!(t0.elapsed());

        println!("{}", res);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_tests() {
        let mut state = State::new();

        let data = fs::read_to_string("tests/combined.ml").unwrap();
        for part in data.split("###") {
            let s = part.trim();
            if s.is_empty() {
                continue;
            }

            let (kind, s) = s.split_once('\n').unwrap();
            // println!("{} {}", kind, s);

            if state.process(s) {
                if kind != "Good" {
                    println!("Unexpectedly passed:\n{}", s);
                    assert_eq!(kind, "Good");
                }
            } else {
                if kind != "Bad" {
                    println!("Unexpected error:\n{}", state.get_err().unwrap());
                    assert_eq!(kind, "Bad");
                }
            };
        }
    }
}
