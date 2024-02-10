Parsa is a functional combinator parsing library. It focuses on consice error handling, modularity, and reusability,
as well as 0-copy string consuption and backtracking.
See [`Parser`] for info on building parsers, and the examples ahead.

# Examples
Lets parse the string "var = 123".
First, our struct and error type:
```rust
use parsa::{ParserString, Parsable};
use parsa::builtins::*;
use parsa::thiserror::Error; //simple errors
use parsa::nevermore::FromNever; //From<Infallible> conversion
use std::num::ParseIntError;

struct Var {
    name: String,
    val: i32,
}
#[derive(Debug, Clone, Error, FromNever)]
enum VarErr {
    #[error("missing var name")]
    Empty(#[from] WordErr),
    #[error("missing \"=\"")]
    MissingEqual(#[from] TakeErr),
    #[error("error parsing number: {0}")]
    NumberParse(#[from] ParseIntError),
}
impl Parsable for Var {
    type Err = VarErr;
    fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
        todo!()
    }
}
```
The first thing we need to do is parse the name of the variable, which we can do with [`word`](crate::builtins::word).
We also want to get rid of whitespace, so we can use [`whitespace`](crate::builtins::whitespace) with the 
[`after`](crate::Parser::after) combinator, as we dont care about its output.
```rust
# use parsa::{ParserString, Parsable, Parser};
# use parsa::builtins::*;
# use parsa::thiserror::Error;
# use parsa::nevermore::FromNever;
# use std::num::ParseIntError;
# struct Var {
#    name: String,
#    val: i32,
# }
# #[derive(Debug, Clone, Error, FromNever)]
# enum VarErr {
#     #[error("missing var name")]
#     Empty(#[from] WordErr),
#     #[error("missing \"=\"")]
#     MissingEqual(#[from] TakeErr),
#     #[error("error parsing number: {0}")]
#     NumberParse(#[from] ParseIntError),
# }
# impl Parsable for Var {
#     type Err = VarErr;
#     fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
let name = word
    .convert_err::<VarErr>() //explicitly set our target error type
                             //not always needed, but usually helps with inference
    .after(whitespace) //whitespace is infallible, so we dont need an explicit variant
                       //in our error type to coerce from it.
    .parse(s)?;
#         todo!()
#     }
# }


```
Next, we want to just make sure that the `=` sign comes after.
```rust
# use parsa::{ParserString, Parsable, Parser};
# use parsa::builtins::*;
# use parsa::thiserror::Error;
# use parsa::nevermore::FromNever;
# use std::num::ParseIntError;
# struct Var {
#    name: String,
#    val: i32,
# }
# #[derive(Debug, Clone, Error, FromNever)]
# enum VarErr {
#     #[error("missing var name")]
#     Empty(#[from] WordErr),
#     #[error("missing \"=\"")]
#     MissingEqual(#[from] TakeErr),
#     #[error("error parsing number: {0}")]
#     NumberParse(#[from] ParseIntError),
# }
# impl Parsable for Var {
#     type Err = VarErr;
#     fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
# let name = word
#    .convert_err::<VarErr>() //explicitly set our target error type
#                             //not always needed, but usually helps with inference
#    .after(whitespace) //whitespace is infallible, so we dont need an explicit variant
#                       //in our error type to coerce from it.
#    .parse(s)?;
let _ = take("=").after(whitespace).parse(s)?; //coerces to MissingEqual
#   todo!()
# }
# }
```
Our final step is to get the number. We will use `word` again, but this time map the result.
```rust
# use parsa::{ParserString, Parsable, Parser};
# use parsa::builtins::*;
# use parsa::thiserror::Error;
# use parsa::nevermore::FromNever;
# use std::num::ParseIntError;
# struct Var {
#    name: String,
#    val: i32,
# }
# #[derive(Debug, Clone, Error, FromNever)]
# enum VarErr {
#     #[error("missing var name")]
#     Empty(#[from] WordErr),
#     #[error("missing \"=\"")]
#     MissingEqual(#[from] TakeErr),
#     #[error("error parsing number: {0}")]
#     NumberParse(#[from] ParseIntError),
# }
# impl Parsable for Var {
#     type Err = VarErr;
#     fn parse(s: &mut ParserString) -> Result<Self, Self::Err> {
# let name = word
#    .convert_err::<VarErr>() //explicitly set our target error type
#                             //not always needed, but usually helps with inference
#    .after(whitespace) //whitespace is infallible, so we dont need an explicit variant
#                       //in our error type to coerce from it.
#    .parse(s)?;
#   let _ = take("=").after(whitespace).parse(s)?; //coerces to MissingEqual
    let val = word
        .convert_err::<VarErr>() //will save headaches
        .and_then(|s| s.parse::<i32>())
        .parse(s)?;
#       todo!()
#     }
# }
```
And now we can build our struct!
```ignore, rust
Ok(Var { name, val })
```
And because this function has the correct signature, it can be used with any method in [`Parser`].
```ignore, rust
let vars: Vec<Var> = Var::parse.many().parse(s)?;
```
