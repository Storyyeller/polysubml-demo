#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod ast;
mod bound_pairs_set;
mod codegen;
mod core;
mod grammar;
mod instantiate;
mod js;
mod parse_types;
mod reachability;
mod spans;
mod type_errors;
mod typeck;
mod unwindmap;
mod utils;

use lasso::Rodeo;
use wasm_bindgen::prelude::*;

use std::fmt::Display;
use std::mem;

use lalrpop_util::ParseError;

use self::codegen::ModuleBuilder;
use self::grammar::ScriptParser;
use self::spans::SpanMaker;
use self::spans::SpanManager;
use self::spans::SpannedError;
use self::typeck::TypeckState;

fn convert_parse_error<T: Display>(mut sm: SpanMaker, e: ParseError<usize, T, (&'static str, spans::Span)>) -> SpannedError {
    match e {
        ParseError::InvalidToken { location } => {
            SpannedError::new1("SyntaxError: Invalid token", sm.span(location, location))
        }
        ParseError::UnrecognizedEof { location, expected } => SpannedError::new1(
            format!(
                "SyntaxError: Unexpected end of input.\nNote: expected tokens: [{}]\nParse error occurred here:",
                expected.join(", ")
            ),
            sm.span(location, location),
        ),
        ParseError::UnrecognizedToken { token, expected } => SpannedError::new1(
            format!(
                "SyntaxError: Unexpected token {}\nNote: expected tokens: [{}]\nParse error occurred here:",
                token.1,
                expected.join(", ")
            ),
            sm.span(token.0, token.2),
        ),
        ParseError::ExtraToken { token } => {
            SpannedError::new1("SyntaxError: Unexpected extra token", sm.span(token.0, token.2))
        }
        ParseError::User { error: (msg, span) } => SpannedError::new1(msg, span),
    }
}

#[wasm_bindgen]
pub struct State {
    parser: ScriptParser,
    spans: SpanManager,
    strings: lasso::Rodeo,

    checker: TypeckState,
    compiler: ModuleBuilder,

    out: Option<String>,
    err: Option<String>,
}
#[wasm_bindgen]
impl State {
    pub fn new() -> Self {
        let mut strings = Rodeo::new();
        let checker = TypeckState::new(&mut strings);

        State {
            parser: ScriptParser::new(),
            spans: SpanManager::default(),
            strings,

            checker,
            compiler: ModuleBuilder::new(),

            out: None,
            err: None,
        }
    }

    fn process_sub(&mut self, source: &str) -> Result<String, SpannedError> {
        let span_maker = self.spans.add_source(source.to_owned());
        let mut ctx = ast::ParserContext {
            span_maker,
            strings: &mut self.strings,
        };

        let ast = self
            .parser
            .parse(&mut ctx, source)
            .map_err(|e| convert_parse_error(ctx.span_maker, e))?;
        let _t = self.checker.check_script(&mut self.strings, &ast)?;

        let mut ctx = codegen::Context(&mut self.compiler, &self.strings);
        let js_ast = codegen::compile_script(&mut ctx, &ast);
        Ok(js_ast.to_source())
    }

    pub fn process(&mut self, source: &str) -> bool {
        let res = self.process_sub(source);
        match res {
            Ok(s) => {
                self.out = Some(s);
                true
            }
            Err(e) => {
                self.err = Some(e.print(&self.spans));
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
        mem::swap(&mut self.checker, &mut TypeckState::new(&mut self.strings));
        mem::swap(&mut self.compiler, &mut ModuleBuilder::new());
    }
}
