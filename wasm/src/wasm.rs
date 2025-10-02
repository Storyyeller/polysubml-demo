#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use compiler_lib::CompilationResult;
use compiler_lib::State as CompilerState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct State {
    s: CompilerState,

    out: Option<String>,
    err: Option<String>,
}
#[wasm_bindgen]
impl State {
    pub fn new() -> Self {
        State {
            s: CompilerState::new(),
            out: None,
            err: None,
        }
    }

    pub fn process(&mut self, source: &str) -> bool {
        let res = self.s.process(source);
        match res {
            CompilationResult::Success(s) => {
                self.out = Some(s);
                true
            }
            CompilationResult::Error(e) => {
                self.err = Some(e);
                false
            }
        }
    }

    pub fn get_output(&mut self) -> Option<String> {
        self.out.take()
    }
    pub fn get_err(&mut self) -> Option<String> {
        self.err.take()
    }

    pub fn reset(&mut self) {
        self.s.reset();
    }
}
