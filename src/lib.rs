#![warn(missing_docs)]
#![doc = include_str!("../docs.md")]

mod parser;
pub use parser::*;

pub mod combinators;
#[cfg(feature = "builtins")] 
pub mod builtins;

///Implicit [`Infallible`] conversions.
///
///[`Infallible`]: std::convert::Infallible
pub use nevermore::FromNever;

#[cfg(test)]
mod tests;

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
    ///
    /////valid utf-8
    ///let mut input = ParserString::from("🗻∈🌏");
    ///assert_eq!(input.take(2), "🗻∈");
    ///assert_eq!(input.take(1), "🌏");
    ///```
    pub fn take(&mut self, n: usize) -> &str {
        let offs = self.get().chars()
            .take(n).map(char::len_utf8).sum();

        let (front, _) = self.get().split_at(offs);

        update(&self.ptr, |ptr| ptr + offs);

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
        if self.ptr.get() + n > self.full.len() {
            return None;
        }

        let offs = self.get().chars()
            .take(n).map(char::len_utf8).sum();

        let (front, _) = self.get().split_at(offs);
        update(&self.ptr, |ptr| ptr + offs);
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
    ///Caller must assure that the resulting pointer lands on a UTF-8 code point.
    ///This library assumes that a function will never add back more than its taken, and thus is
    ///considered undefined behavior. This will never cause memory-unsafety, but can cause
    ///unpredictable things to happen.
    pub unsafe fn give(&mut self, n: usize) {
        *self.ptr.get_mut() -= n;
    }

    ///Set the current start position manually.
    ///# Safety
    ///Caller must assure that the resulting pointer lands on a UTF-8 code point.
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
        self.full.len() - self.ptr.get()
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
        Self {
            full: Box::from(value),
            ptr: Cell::new(0),
        }
    }
}

impl From<String> for ParserString {
    fn from(value: String) -> Self {
        Self {
            full: value.into_boxed_str(),
            ptr: Cell::new(0),
        }
    }
}
