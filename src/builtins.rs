/*!
Composable parsers for common actions.

See the [error coercion rules](crate::combinators#error-coercion-rules) for error handling.
*/


use std::{convert::Infallible, str::FromStr};

use thiserror::Error;
use nevermore::FromNever;

use crate::{ParserString, Parser};

/**
Returns the next character in the string, `Err(())` if the string is empty.
```
# use parsa::ParserString;
# use parsa::builtins::next;
let mut input = ParserString::from("abc");

assert_eq!(next(&mut input), Ok('a'));
assert_eq!(next(&mut input), Ok('b'));
assert_eq!(next(&mut input), Ok('c'));
assert_eq!(next(&mut input), Err(()));
```
*/
pub fn next(s: &mut ParserString) -> Result<char, ()> {
    s.try_take(1).ok_or(())?.chars().next().ok_or(())
}

/**
Returns the next string of characters up until whitespace, returning [`WordErr`] if the next character is whitespace.
```
# use parsa::ParserString;
# use parsa::builtins::word;
let mut input = ParserString::from("abc 123");

assert!(word(&mut input).is_ok_and(|s| s == "abc"));
input.take(1);
assert!(word(&mut input).is_ok_and(|s| s == "123"));
assert!(word(&mut input).is_err());
```
*/
pub fn word(s: &mut ParserString) -> Result<String, WordErr> {
    let mut out = String::new(); 

    while let Ok(c) = next(s) {
        if !c.is_whitespace() {
            out.push(c);
        } else {
            unsafe { s.give(1) }
            break;
        }
    }
    
    if out.len() == 0 { return Err(WordErr) }
    Ok(out)
}
///Indicates that a [`word`] parser has failed.
#[derive(Debug, Clone, Copy, Error, FromNever)]
#[error("found no characters")]
pub struct WordErr;

/**Removes leading whitespace in string, returning the amount. 

This function returns [`Infallible`]
as its error type, and thus can never fail. If you derive [`FromNever`], this type will coerce
implicitly.
```
# use parsa::ParserString;
# use parsa::Parser;
# use parsa::builtins::whitespace;
let mut input = ParserString::from("    abc");
let ctr = whitespace(&mut input).unwrap(); // function can never fail
assert_eq!(ctr, 4);
# let ctr = whitespace(&mut input).unwrap(); // function can never fail
# assert_eq!(input.get(), "abc");
```
*/
pub fn whitespace(s: &mut ParserString) -> Result<usize, Infallible> {
    let mut ctr = 0;

    while let Ok(c) = next.parse(s) { 
        if c != ' ' {
            break
        }
        ctr += 1 
    }

    if !s.get().is_empty() {
        unsafe { s.give(1) }
    }
    Ok(ctr)
}


/**Take the delimiter from the front of the string.
```
# use parsa::ParserString;
# use parsa::Parser;
# use parsa::builtins::take;
let mut input = ParserString::from("abc 123");

let head = take("ab").parse(&mut input);

assert!(head.is_ok_and(|s| s == "ab"));
assert_eq!(input.get(), "c 123");
```
*/
pub fn take(delim: &'static str) -> impl Parser<&'static str, Err = TakeErr> {
    move |s: &mut ParserString| {
        let head = s.try_take(delim.len())
            .ok_or(TakeErr::NoSpace)?;

        if head == delim {
            Ok(delim)
        } else {
            Err(TakeErr::NoMatch)
        }
    }
}

///Indicates that a [`take`] parser has failed.
#[derive(Debug, Clone, Copy, Error, FromNever)]
pub enum TakeErr {
    ///Parser failed because the string ended
    #[error("ran out of space")]
    NoSpace,
    ///Parser failed because the captured slice didn't match the delimiter
    #[error("did not match delim")]
    NoMatch,
}

///Indicates that an [`int`] parser has failed.
#[derive(Debug, Clone, Copy, Error, FromNever)]
pub enum IntErr<E: std::error::Error> {
    ///Parser failed from a [`WordErr`]
    #[error("{0}")]
    Word(#[from] WordErr), 
    ///Parser failed from a [`FromStr`] error
    #[error("error parsing int: {0}")]
    Parse(E)
}
/**Parses a [`word`] into an integer.
```
# use parsa::ParserString;
# use parsa::Parser;
# use parsa::builtins::int;
let mut input = ParserString::from("123");

let num = int::<i32, _>(&mut input);
assert!(num.is_ok_and(|i| i == 123));
```
*/
pub fn int<I, E>(s: &mut ParserString) -> Result<I, IntErr<E>> 
where I: num_traits::PrimInt + FromStr<Err = E> + 'static, E: std::error::Error + 'static
{
    word
    .convert_err::<IntErr<E>>()
    .and_then(|s| {
        s.parse::<I>()
        .map_err(|e| IntErr::Parse(e))
    })
    .parse(s)
}

///Indicates that an [`float`] parser has failed.
#[derive(Debug, Clone, Copy, Error, FromNever)]
pub enum FloatErr<E: std::error::Error> {
    ///Parser failed from a [`WordErr`]
    #[error("{0}")]
    Word(#[from] WordErr), 
    ///Parser failed from a [`FromStr`] error
    #[error("error parsing int: {0}")]
    Parse(E)
}
/**Parses a [`word`] into a float.
```
# use parsa::ParserString;
# use parsa::Parser;
# use parsa::builtins::float;
let mut input = ParserString::from("123.4");

let num = float::<f32, _>(&mut input);
assert!(num.is_ok_and(|i| i == 123.4));
```
*/
pub fn float<I, E>(s: &mut ParserString) -> Result<I, FloatErr<E>> 
where I: num_traits::Float + FromStr<Err = E> + 'static, E: std::error::Error + 'static
{
    word
    .convert_err::<FloatErr<E>>()
    .and_then(|s| {
        s.parse::<I>()
        .map_err(|e| FloatErr::Parse(e))
    })
    .parse(s)
}

///Indicates that a [`between`] parser has failed.
#[derive(Debug, Clone, Copy, Error, FromNever)]
pub enum BetweenErr {
    ///Parser failed because the opener was not found
    #[error("opener was not found")] 
    NoOpen,
    ///Parser failed because the closer was not found
    #[error("string ended before closer was found")] 
    Unmatched,
}
/**Takes a segment between a given opener and closer.
```
# use parsa::ParserString;
# use parsa::Parser;
# use parsa::builtins::{next, between};
let mut input = ParserString::from("(abc) ");
let middle = between("(", ")").parse(&mut input);
assert!(middle.is_ok_and(|s| s == "abc"));
# assert!(next(&mut input).is_ok_and(|c| c == ' '));
```
*/
pub fn between(open: &'static str, close: &'static str) -> impl Parser<String, Err = BetweenErr> {
    move |s: &mut ParserString| {
        let _ = take(open).map_err(|_| BetweenErr::NoOpen).parse(s)?;
        let mut out = String::with_capacity(s.len());
        
        while take(close).try_parse(s).is_err() {
            out.push(next(s).map_err(|_| BetweenErr::Unmatched)?);
        }

        Ok(out)
    }
}
