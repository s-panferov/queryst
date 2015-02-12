#![feature(collections)]
#![feature(core)]
#![feature(plugin)]
#![plugin(regex_macros)]
extern crate "rustc-serialize" as serialize;

extern crate regex_macros;
extern crate regex;

extern crate collections;
extern crate url;

pub use parser::{parse, ParseResult, ParseError, ParseErrorKind};

mod merge;
mod mutable_json;
mod helpers;
mod parser;

