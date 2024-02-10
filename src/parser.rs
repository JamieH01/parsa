use crate::{combinators::*, ParserString};

use paste::paste;

macro_rules! delegate {
    (
        [$($dec_gen:ident $(: $rest:ty)?),*] //generics
        $name: ident<$($s_gen:ty),*>, //struct
        (self, $($arg:ident: $arg_ty: ty),*)) => { //arg
    paste! {
        #[doc = concat!("Constructs a [`", stringify!($name), "`] combinator.")] 
        fn [<$name:snake>]<$($dec_gen $(: $rest)?),*>(self, $($arg:$arg_ty),*) -> $name<$($s_gen),*> {
            $name::new(self, $($arg),*)
        } 
    } 
    };
}


///All parsers implement this trait. Any function or closure with the signature 
///`Fn(&mut ParserString) -> Result<T, E>` implements Parser.
pub trait Parser<T>: Sized {
    ///The error type this parser can return
    type Err;
    ///Run this parser, using a [`ParserString`].
    fn parse(&self, s: &mut ParserString) -> Result<T, Self::Err>;

    ///Run this parser without affecting the string on failure. In other words, the string will be
    ///"rewinded" on failure.
    fn try_parse(&self, s: &mut ParserString) -> Result<T, Self::Err> {
        let i = s.start();
        self.parse(s).map_err(|err| {
            unsafe { s.set_ptr(i) };
            err
        })
    }

    delegate! {
        [U, P2: Parser<U, Err = E>, E: Into<Self::Err>] 
        Chain<T, U, Self, P2>, 
        (self, other: P2)
    } 

    delegate! {
        [P2: Parser<T, Err = E>, E: Into<Self::Err>]
        Or<T, E, Self, P2>, 
        (self, other: P2)
    }

    delegate! {
        []
        Many<T, Self>,
        (self, )
    }

    ///Apply a function to the output of this parser on success.
    fn map<U: 'static>(self, f: impl Fn(T) -> U + 'static) -> impl Parser<U, Err = Self::Err> {
        move |s: &mut ParserString| {
            self.parse(s).map(&f)
        }
    }
    ///Apply a function to the [`Err`] output of this parser on failure.
    fn map_err<E: 'static>(self, f: impl Fn(Self::Err) -> E + 'static) -> impl Parser<T, Err = E> {
        move |s: &mut ParserString| {
            self.parse(s).map_err(&f)
        }
    }
    ///Applies a function to the output of this parser on success, using [error coercion rules](crate::combinators#error-coercion-rules).
    fn and_then<U: 'static, E: Into<Self::Err>>(self, f: impl Fn(T) -> Result<U, E> + 'static) -> impl Parser<U, Err = Self::Err> {
        move |s: &mut ParserString| -> Result<U, Self::Err> {
            match self.parse(s) {
                Ok(v) => f(v).map_err(Into::into),
                Err(e) => Err(e),
            }
        }
    }

    ///Similar to [`Chain`], but only keeps the output of the first parser.
    fn after<U, P2: Parser<U, Err = E>, E: Into<Self::Err>>(self, other: P2) -> impl Parser<T, Err = Self::Err> {
        let p = self.chain(other);
        move |s: &mut ParserString| {
            p.parse(s).map(|(x, _)| x)
        }
    }

    ///Similar to [`Chain`], but only keeps the output of the second parser.
    fn replace<U, P2: Parser<U, Err = E>, E: Into<Self::Err>>(self, other: P2) -> impl Parser<U, Err = Self::Err> {
        let p = self.chain(other);
        move |s: &mut ParserString| {
            p.parse(s).map(|(_, x)| x)
        }
    }

    ///Explicitly sets the target error type of this parser. Can help with type inference.
    fn convert_err<E: From<Self::Err> + 'static>(self) -> impl Parser<T, Err = E> {
        self.map_err(|e| e.into())
    }
}

///Parse an instance of this type, Similar to [`FromStr`].
pub trait Parsable: Sized {
    ///The error type this parser can return
    type Err;
    ///Run this parser, using a [`ParserString`].
    fn parse(s: &mut ParserString) -> Result<Self, Self::Err>;

    ///Run this parser without affecting the string on failure. In other words, the string will be
    ///"rewinded" on failure.
    fn try_parse(s: &mut ParserString) -> Result<Self, Self::Err> {
        Self::parse.try_parse(s)
    }
}

impl<T, E, F: Fn(&mut ParserString) -> Result<T, E>> Parser<T> for F {
    type Err = E;
    fn parse(&self, s: &mut ParserString) -> Result<T, Self::Err> {
        self(s)
    }
}
