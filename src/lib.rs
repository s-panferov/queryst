#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate collections;
extern crate url;

pub use parser::{parse, ParseResult, ParseError, ParseErrorKind};

mod merge;
mod mutable_json;
mod helpers;
mod parser;

