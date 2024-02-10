/*!
Parsers that combine/manipulate other parsers in some way.

All of the types here have corresponding methods in [`Parser`] for builder pattern-style construction.

# Error coercion rules
When dealing with multiple parsers with different error types, they must have a conversion to a single type.
This is usually determined by the first error type introduced, called the "target" type. The [`Parser::convert_err`] method
can be used to set this target type explicitly.

For example with [`Chain`], `P2::Err` must implement `Into<P1::Err>`.
This is usually achieved with a higher ranked error type that fits the context of its usage.
```ignore
#[derive(Debug, Error)]
pub enum MyErr {
    Unique,
    Empty(#[from] WordErr), // <- WordErr will implicitly be elevated to this type
}
```
*/

use std::{marker::PhantomData, convert::Infallible};

use crate::{Parser, ParserString};

/**Chains two parsers together.

Follows [error coercion rules](crate::combinators#error-coercion-rules).
```
# use parsa::{ParserString, Parser};
# use parsa::builtins::*;
# fn main() -> Result<(), WordErr> {
let mut input = ParserString::from("abc   ");
let (string, after) = word.chain(whitespace).parse(&mut input)?;

assert_eq!(string, "abc");
assert_eq!(after, 3);
# Ok(())
# }
```
*/
pub struct Chain<T, U, P1, P2> 
{
    p1: P1,
    p2: P2,

    t: PhantomData<T>,
    u: PhantomData<U>,
}

impl<T, U, P1, P2, E> Chain<T, U, P1, P2>
where 
    P1: Parser<T>,
    E: Into<P1::Err>,
    P2: Parser<U, Err = E>,
{
    ///Constructs this parser.
    pub fn new(p1: P1, p2: P2) -> Self { Self { p1, p2, t: PhantomData, u: PhantomData } }
}

impl<T, U, P1, P2, E> Parser<(T, U)> for Chain<T, U, P1, P2>
where 
    P1: Parser<T>,
    E: Into<P1::Err>,
    P2: Parser<U, Err = E>,
{
    type Err = P1::Err;

    fn parse(&self, s: &mut ParserString) -> Result<(T, U), Self::Err> {
        Ok((
            self.p1.parse(s)?, 
            self.p2.parse(s).map_err(|e| e.into())?
        ))
    }
}

/**
Attempts a second parser.

```
# use parsa::{Parser, Parsable};
# use parsa::ParserString;
# use parsa::builtins::{take, TakeErr};
# #[derive(Debug, PartialEq, Eq)]
struct Abc;
# #[derive(Debug, PartialEq, Eq)]
struct Def;
# #[derive(Debug, PartialEq, Eq)]
enum Tag {
    Abc(Abc),
    Def(Def),
}
impl From<Abc> for Tag 
# {
#    fn from(value: Abc) -> Tag {
#        Tag::Abc(value)
#    }
# }
impl From<Def> for Tag 
# {
#    fn from(value: Def) -> Tag {
#        Tag::Def(value)
#    }
# }

impl Parsable for Abc {
    type Err = TakeErr;
    fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
        take("abc").map(|_| Abc).parse(s)
    }
}
impl Parsable for Def {
    type Err = TakeErr;
    fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
        take("def").map(|_| Def).parse(s)
    }
}

impl Parsable for Tag {
    type Err = ();
    fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
        Abc::parse.map(Tag::from)
        .or(Def::parse.map(Tag::from))
        .map_err(|_| ())
        .parse(s)
    }
}

let mut input = ParserString::from("abcdef");
assert!(Tag::parse(&mut input).is_ok_and(|t| t == Tag::Abc(Abc)));
assert!(Tag::parse(&mut input).is_ok_and(|t| t == Tag::Def(Def)));
```
*/
pub struct Or<T, E, P1, P2> 
where 
    P1: Parser<T>,
    E: Into<P1::Err>,
    P2: Parser<T, Err = E>
{
    p1: P1,
    p2: P2,
    t: PhantomData<T>,  
    e: PhantomData<E>  
}

impl<T, E, P1, P2> Or<T, E, P1, P2>
where 
    P1: Parser<T>,
    E: Into<P1::Err>,
    P2: Parser<T, Err = E>
{
    ///Constructs this parser.
    pub fn new(p1: P1, p2: P2) -> Self { Self { p1, p2, t: PhantomData, e: PhantomData } }
}

impl<T, E, P1, P2> Parser<T> for Or<T, E, P1, P2>
where 
    P1: Parser<T>,
    E: Into<P1::Err>,
    P2: Parser<T, Err = E>
{
    type Err = P1::Err;

    fn parse(&self, s: &mut ParserString) -> Result<T, Self::Err> {
        match self.p1.try_parse(s) {
            Ok(v) => Ok(v),
            Err(_) => self.p2.parse(s).map_err(Into::into),
        }
    }
}

/**
Repeatedly applies a parser, until it fails.

```
# use parsa::builtins::{word, WordErr, whitespace};
# use parsa::{ParserString, Parser};
let mut input = ParserString::from("ab cd ef gh");
let words = word.after(whitespace).many().parse(&mut input).unwrap();
assert_eq!(words, vec!["ab", "cd", "ef", "gh"]);
```
*/
pub struct Many<T, P> 
where 
    P: Parser<T>
{
    p: P,
    t: PhantomData<T>
}

impl<T, P> Many<T, P>
where 
    P: Parser<T>
{
    ///Constructs this parser.
    pub fn new(p: P) -> Self { Self { p, t: PhantomData } }
}

impl<T, P> Parser<Vec<T>> for Many<T, P>
where 
    P: Parser<T>
{
    type Err = Infallible;

    fn parse(&self, s: &mut ParserString) -> Result<Vec<T>, Self::Err> {
        let mut out = vec![];
        
        while let Ok(v) = self.p.try_parse(s) {
            out.push(v)
        }

        Ok(out)
    }
}
