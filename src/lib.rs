extern crate rustc_serialize as serialize;

extern crate regex;
extern crate url;

#[macro_use]
extern crate lazy_static;

pub use parser::{parse, ParseResult, ParseError, ParseErrorKind};

mod merge;
mod mutable_json;
mod helpers;
mod parser;

