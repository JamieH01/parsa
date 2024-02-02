#![warn(missing_docs)]

//! crate-level docs

mod parser;
pub use crate::parser::*;

pub mod combinators;
pub mod builtins;

mod result;
pub use crate::result::*;

use std::cell::Cell;
///A shrinking-window read-only string. 
///
///String slices can be taken from the front, and reset, with zero
///allocations or copies.
pub struct ParserString {
    full: Box<str>,
    ptr: Cell<usize>,
}

fn update<T: Copy, F: Fn(T) -> T>(cell: &Cell<T>, f: F) {
    let a = cell.get();
    cell.set(f(a));
}

impl ParserString {
    ///Splits the string at `n`, shrinking it. Panics if `n` is larger than the remaining slice.
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///
    ///assert_eq!(input.take(3), "abc");
    ///assert_eq!(input.take(3), "123");
    ///```
    pub fn take(&mut self, n: usize) -> &str {
        let (front, _) = self.get().split_at(n);

        update(&self.ptr, |ptr| ptr + n);

        assert!(self.ptr.get() <= self.full.len());

        front
    }

    ///Splits the string at `n`, shrinking it. Returns [`None`] if `n` is larger than the remaining slice.
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///assert_eq!(input.try_take(5), Some("abc12"));
    ///assert_eq!(input.try_take(5), None);
    ///
    ///```
    pub fn try_take(&mut self, n: usize) -> Option<&str> {
        if self.ptr.get()+n >= self.full.len() {
            return None
        }

        let (front, _) = self.get().split_at(n);
        update(&self.ptr, |ptr| ptr + n);
        Some(front)
    }


    ///Rewinds the string slice `n` spaces. Panics if `n` is larger than the taken space.    
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///
    ///assert_eq!(input.take(3), "abc");
    ///
    ///unsafe { input.give(3); }
    ///
    ///assert_eq!(input.take(3), "abc");
    ///assert_eq!(input.take(3), "123");
    ///```
    ///# Safety
    ///This library assumes that a function will never add back more than its taken, and thus is
    ///considered undefined behavior. This will never cause memory-unsafety, but can cause
    ///unpredictable things to happen.
    pub unsafe fn give(&mut self, n: usize) {
        *self.ptr.get_mut() -= n;
    }

    ///Set the current start position manually.
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///unsafe { input.set_ptr(3); }
    ///assert_eq!(input.get(), "123");
    ///```
    pub unsafe fn set_ptr(&mut self, ptr: usize) {
        self.ptr.set(ptr);
    }

    ///Get a reference to the string slice.
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///let _ = input.take(2);
    ///
    ///assert_eq!(input.get(), "c123");
    ///```
    pub fn get(&self) -> &str {
        &self.full[self.ptr.get()..]
    }

    ///Get the length of the string.
    ///```rust
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///let _ = input.take(2);
    ///assert_eq!(input.len(), 4);
    ///```
    pub fn len(&self) -> usize {
        self.full.len()-self.ptr.get()
    }

    ///Get the current start of the string, relative to the "true" start.
    ///```rust 
    ///# use parsa::ParserString;
    ///let mut input = ParserString::from("abc123"); 
    ///let _ = input.take(2);
    ///assert_eq!(input.start(), 2);
    ///```
    pub fn start(&self) -> usize {
        self.ptr.get()
    }
}

impl From<&str> for ParserString {
    fn from(value: &str) -> Self {
        Self { full: Box::from(value), ptr: Cell::new(0) }
    }
}

impl From<String> for ParserString {
    fn from(value: String) -> Self {
        Self { full: value.into_boxed_str(), ptr: Cell::new(0) }
    }
}

#[macro_export]
///A macro to emulate `?` on [`ParseResult`]. 
///
///Returns the [`Ok`] variant, propogating
///[`Recoverable`]
///and [`Unrecoverable`]
///
///```rust
///# use parsa::*;
///# fn parse<T, U, E>(p1: impl Parser<T, E>, p2: impl Parser<U, E>, s: &mut ParserString) -> ParseResult<(T, U), E> {
///let a = p_try!(p1.parse(s));
///let b = p_try!(p2.parse(s));
///ParseResult::Ok((a, b))
///# }
///```
///
/// [`Ok`]: ParseResult::Ok
/// [`Recoverable`]: ParseResult::Recoverable
/// [`Unrecoverable`]: ParseResult::Recoverable
macro_rules! p_try {
    ($expr:expr) => {
        match $expr {
            ParseResult::Ok(item) => item,
            ParseResult::Recoverable(e) => return ParseResult::Recoverable(e),
            ParseResult::Unrecoverable(e) => return ParseResult::Unrecoverable(e),
        } 
    };
}

#[macro_export]
///A macro to emulate `?` on [`ParseResult`]. 
///
///Returns a [`Result`] with the [`Ok`] and [`Recoverable`]
///variants, propogating [`Unrecoverable`]
///```rust
///# use parsa::*;
///# fn parse<T: Default, U, E>(p1: impl Parser<T, E>, s: &mut ParserString) -> ParseResult<T, E> {
///let res = match p_try_recover!(p1.parse(s)) {
///    Ok(v) => v,
///    Err(_) => T::default(), //recoverable variant
///};
///ParseResult::Ok(res)
///# }
///```
///
/// [`Ok`]: ParseResult::Ok
/// [`Recoverable`]: ParseResult::Recoverable
/// [`Unrecoverable`]: ParseResult::Recoverable
macro_rules! p_try_recover {
    ($expr:expr) => {
        match $expr {
            ParseResult::Ok(item) => Ok(item),
            ParseResult::Recoverable(e) => Err(e),
            ParseResult::Unrecoverable(e) => return ParseResult::Unrecoverable(e),
        } 
    };
}
