use std::{convert::Infallible, fmt::Display};

use super::{from_index, Letter};

/// A helper function that combines consecutive, ascending numbers into ranges.
fn combine_into_ranges(
  nums: impl IntoIterator<Item = usize>,
) -> impl Iterator<Item = (usize, usize)> {
  let mut nums = nums.into_iter();
  let mut range = nums.next().map(|n| (n, n));
  nums
    .filter_map(move |n| {
      if let Some((s, e)) = range {
        if e + 1 == n {
          range.replace((s, n));
          return None;
        }
      }
      range.replace((n, n))
    })
    .chain(range)
}

// We're going to try to closely mimic nom's parser combinators here.

#[derive(Debug, Clone)]
enum ErrorKind {
  TooManyChars(usize),
  TooFewChars(usize),
}

fn unparse_dot_letter(Letter(mask): &Letter) -> Result<String, ErrorKind> {
  if mask == &[true; 26] {
    return Ok(".".into());
  }

  let num_chars = mask.iter().filter(|&&b| b).count();
  Err(ErrorKind::TooFewChars(num_chars))
}

fn unparse_char_letter(Letter(mask): &Letter) -> Result<String, ErrorKind> {
  if mask == &[false; 26] {
    return Err(ErrorKind::TooFewChars(0));
  }

  let num_chars = mask.iter().filter(|&&b| b).count();
  if num_chars > 1 {
    return Err(ErrorKind::TooManyChars(num_chars));
  }

  let idx = mask.iter().position(|&b| b).unwrap();
  Ok(from_index(idx).into())
}

fn unparse_char_pair(&(start, end): &(char, char)) -> Result<String, Infallible> {
  if start == end {
    Ok(start.into())
  } else {
    Ok(format!("{}-{}", start, end))
  }
}

fn unparse_group_letter(letter: &Letter) -> Result<String, Infallible> {
  let ranges = combine_into_ranges(letter.indices());
  let ranges_string = ranges
    .map(|(s, e)| unparse_char_pair(&(from_index(s), from_index(e))))
    .collect::<Result<String, _>>()?;
  Ok(format!("[{}]", ranges_string))
}

pub(crate) fn unparse_letter(letter: &Letter) -> Result<String, Infallible> {
  unparse_dot_letter(letter)
    .or_else(|_| unparse_char_letter(letter))
    .or_else(|_| unparse_group_letter(letter))
}

impl Display for Letter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", unparse_letter(self).unwrap())
  }
}
