use super::{into_index, Letter};
use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::{anychar, char},
  combinator::{map_res, opt, verify},
  error::Error,
  multi::many1,
  sequence::delimited,
  Finish, IResult,
};
use std::{ops::RangeInclusive, str::FromStr};

impl Letter {
  pub(crate) fn all() -> Self {
    Self([true; 26])
  }

  // todo: `String` is a bad error type, define a better one
  pub(crate) fn try_from_alpha(c: char) -> Result<Self, String> {
    if !c.is_ascii_lowercase() {
      return Err(format!("`{c}` is not in a-z"));
    }
    let mut mask = [false; 26];
    mask[into_index(c)] = true;
    Ok(Self(mask))
  }

  pub(crate) fn try_from_iter(iter: impl IntoIterator<Item = char>) -> Result<Self, String> {
    let mut mask = [false; 26];
    for c in iter {
      if !c.is_ascii_lowercase() {
        return Err(format!("`{c}` is not in a-z"));
      }
      mask[into_index(c)] = true;
    }
    Ok(Self(mask))
  }
}

fn parse_lower_char(input: &str) -> IResult<&str, char> {
  verify(anychar, char::is_ascii_lowercase)(input)
}

fn parse_dot_letter(input: &str) -> IResult<&str, Letter> {
  let (input, _) = char('.')(input)?;
  Ok((input, Letter::all()))
}

fn parse_char_letter(input: &str) -> IResult<&str, Letter> {
  map_res(parse_lower_char, Letter::try_from_alpha)(input)
}

fn parse_single_char_range(input: &str) -> IResult<&str, RangeInclusive<char>> {
  let (input, c) = parse_lower_char(input)?;
  Ok((input, c..=c))
}

fn parse_multi_char_range(input: &str) -> IResult<&str, RangeInclusive<char>> {
  let (input, start) = opt(parse_lower_char)(input)?;
  let (input, _) = tag("-")(input)?;
  let (input, end) = opt(parse_lower_char)(input)?;
  Ok((input, start.unwrap_or('a')..=end.unwrap_or('z')))
}

fn parse_char_range(input: &str) -> IResult<&str, RangeInclusive<char>> {
  alt((parse_multi_char_range, parse_single_char_range))(input)
}

fn parse_group_letter(input: &str) -> IResult<&str, Letter> {
  map_res(many1(parse_char_range), |ranges| {
    Letter::try_from_iter(ranges.into_iter().flatten())
  })(input)
}

pub(crate) fn parse_letter(input: &str) -> IResult<&str, Letter> {
  alt((
    parse_dot_letter,
    parse_char_letter,
    delimited(char('['), parse_group_letter, char(']')),
  ))(input)
}

impl FromStr for Letter {
  type Err = Error<String>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match parse_letter(s).finish() {
      Ok((_, l)) => Ok(l),
      Err(Error { input, code }) => Err(Error {
        input: input.to_string(),
        code,
      }),
    }
  }
}
