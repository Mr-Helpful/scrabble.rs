use super::Word;
use crate::letter::parse::parse_letter;
use nom::{combinator::map, error::Error, multi::many0, Finish, IResult};
use std::str::FromStr;

pub(crate) fn parse_word(input: &str) -> IResult<&str, Word> {
  map(many0(parse_letter), |letters| letters.into_iter().collect())(input)
}

impl FromStr for Word {
  type Err = Error<String>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match parse_word(s).finish() {
      Ok((_, word)) => Ok(word),
      Err(Error { input, code }) => Err(Error {
        input: input.to_string(),
        code,
      }),
    }
  }
}
