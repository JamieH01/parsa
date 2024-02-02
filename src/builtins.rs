//!Simple built-in parsers.

use crate::Parser;
use crate::ParseResult;
use crate::ParserString;

pub fn take<'a, E, P>(delim: &'static str, err: E) -> impl for<'b> Parser<&'a str, E> {
    Box::new(move |s: &'a mut ParserString| {
        match s.try_take(delim.len()) {
            Some(v) if v == delim => ParseResult::Ok(v),
            _ => ParseResult::Recoverable(err),
        }
    })
}
