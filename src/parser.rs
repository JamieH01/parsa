use crate::{ParserString, ParseResult, combinators::{Map, AndThen, Try} };

///Trait representing a parser. 
///
///Implemented for all [`combinators`], as well as [`FnOnce`] given it has the same
///signature as [`parse`].
///
///[`parse`]: Parser::parse
///[`combinators`]: crate::combinators
pub trait Parser<T, E>: Sized {
    ///Run this parser
    fn parse(self, s: &mut ParserString) -> ParseResult<T, E>;

    ///Builds a [`Map`] combinator.
    fn map<U>(self, f: impl FnOnce(T) -> U) 
    -> Map<T, U, E, Self, impl FnOnce(T) -> U> {
        Map::new(self, f)
    }

    ///Builds an [`AndThen`] combinator.
    fn and_then<U>(self, p: impl Parser<U, E>)
    -> AndThen<T, U, E, Self, impl Parser<U, E>> {
        AndThen::new(self, p)
    }

    ///Builds a [`Try`] combinator.
    fn rewind(self) -> Try<T, E, Self> {
        Try::new(self)
    }
}

//base parser
impl<T, E, F: FnOnce(&mut ParserString) -> ParseResult<T, E>> Parser<T, E> for Box<F> {
    fn parse(self, s: &mut ParserString) -> ParseResult<T, E> {
        (self)(s) 
    }
}
