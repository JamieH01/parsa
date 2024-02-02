//!Types that combine parsers.
//!
//!Every combinator has a coresponding method in [`Parser`] to construct them.

use std::marker::PhantomData;

use crate::{ParserString, ParseResult, p_try, Parser, p_try_recover};

///Apply a function to the output of a parser, given it succeeds.
pub struct Map<T, U, E, P: Parser<T, E>, F: FnOnce(T) -> U> {
    p: P,
    f: F, 

    t: PhantomData<T>,
    e: PhantomData<E>,
}

impl<T, U, E, P: Parser<T, E>, F: FnOnce(T) -> U> Map<T, U, E, P, F> {
    ///Constructs this combinator.
    pub fn new(p: P, f: F) -> Self { Self { p, f, t: PhantomData, e: PhantomData } }
}

impl<T, U, E, P: Parser<T, E>, F: FnOnce(T) -> U> Parser<U, E> for Map<T, U, E, P, F> {
    fn parse(self, s: &mut ParserString) -> ParseResult<U, E> {
        self.p.parse(s).map(self.f)
    }
}

///Runs another parser if the first succeeded, returning `(T, U)` with both outputs.
pub struct AndThen<T, U, E, P1: Parser<T, E>, P2: Parser<U, E>> {
    p1: P1,
    p2: P2,


    t: PhantomData<T>,
    u: PhantomData<U>,
    e: PhantomData<E>,
}

impl<T, U, E, P1: Parser<T, E>, P2: Parser<U, E>> AndThen<T, U, E, P1, P2> {
    ///Constructs this combinator.
    pub fn new(p1: P1, p2: P2) -> Self { Self { p1, p2, t: PhantomData, u: PhantomData, e: PhantomData } }
} 

impl<T, U, E, P1: Parser<T, E>, P2: Parser<U, E>> Parser<(T, U), E> for AndThen<T, U, E, P1, P2> {
    fn parse(self, s: &mut ParserString) -> ParseResult<(T, U), E> {
        let a = p_try!(self.p1.parse(s));
        let b = p_try!(self.p2.parse(s));
        ParseResult::Ok((a, b))
    }
}

///Restores the input string to its previous state on [`Recoverable`] failure.
///
///[`Recoverable`]: ParseResult::Recoverable
pub struct Try<T, E, P: Parser<T, E>> {
    p: P,

    t: PhantomData<T>,
    e: PhantomData<E>,
}

impl<T, E, P: Parser<T, E>> Try<T, E, P> {
    ///Constructs this combinator.
    pub fn new(p: P) -> Self { Self { p, t: PhantomData, e: PhantomData } }
}

impl<T, E, P: Parser<T, E>> Parser<T, E> for Try<T, E, P> {
    fn parse(self, s: &mut ParserString) -> ParseResult<T, E> {
        let save = s.start();
        let res = self.p.parse(s);

        if res.is_recoverable() {
            unsafe { s.set_ptr(save) }
        }

        res
    }
}

///Try first parser, rewinding and trying the second if it fails.
pub struct Or<T, E, P1: Parser<T, E>, P2: Parser<T, E>> {
    p1: P1,
    p2: P2,

    t: PhantomData<T>,
    e: PhantomData<E>,
}

impl<T, E, P1: Parser<T, E>, P2: Parser<T, E>> Or<T, E, P1, P2> {
    ///Constructs this combinator.
    pub fn new(p1: P1, p2: P2) -> Self { Self { p1, p2, t: PhantomData, e: PhantomData } }
}

impl<T, E, P1: Parser<T, E>, P2: Parser<T, E>> Parser<T, E> for Or<T, E, P1, P2> {
    fn parse(self, s: &mut ParserString) -> ParseResult<T, E> {
        match p_try_recover!(self.p1.rewind().parse(s)) {
            Ok(v) => ParseResult::Ok(v),
            Err(_) => self.p2.parse(s),
        }
    }
}
