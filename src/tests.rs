//"name = 123"

use std::num::ParseIntError;

use crate::{Parsable, builtins::{whitespace, take, TakeErr, word, WordErr}, Parser, ParserString};
use nevermore::FromNever;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
struct Var {
    name: String,
    val: i32,
}

#[derive(Debug, Error, FromNever)]
enum VarErr {
    #[error("")]
    Take(#[from] TakeErr),
    #[error("")]
    Word(#[from] WordErr),
    #[error("")]
    ParseInt(#[from] ParseIntError),
}
impl Parsable for Var {
    type Err = VarErr;

    fn parse(s: &mut crate::ParserString) -> Result<Self, Self::Err> {
        let name = word.convert_err::<VarErr>()
            .after(whitespace)
            .after(take("=").after(whitespace))
            .parse(s)?;
        let val = word.convert_err::<VarErr>()
            .and_then(|s| s.parse::<i32>())
            .parse(s)?;
        Ok(Self { name, val })
    }
}

#[test]
fn var_parse_test() {
    let mut inp = ParserString::from("val = 123");
    let res = Var::parse(&mut inp).unwrap();
    assert_eq!(Var { name: "val".to_owned(), val: 123 }, res);
}