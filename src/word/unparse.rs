use super::Word;
use crate::letter::unparse::unparse_letter;
use std::{convert::Infallible, fmt::Display};

fn unparse_word(word: &Word) -> Result<String, Infallible> {
  word
    .letters()
    .map(unparse_letter)
    .collect::<Result<String, _>>()
}

impl Display for Word {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", unparse_word(self).unwrap())
  }
}
