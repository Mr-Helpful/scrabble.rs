/*!
A custom regex-like format for specifying words of fixed length

# Scope

Allowed:
- character groups, open and closed ranges, i.e.
  - `[abc]`
  - `[a-d] = [abcd]`
  - `[-f] = [abcdef]`
  - `[w-] = [wxyz]`

- `.` character to represent all letters, i.e.
  - `. = [-] = [a-z] = [abcdefghijklmnopqrstuvwxyz]`

- standard alphabetic characters, i.e.
  - only `abcdefghijklmnopqrstuvwxyz`

Disallowed:
- non-alphabetic characters, e.g.
  - `5`, `!` or `}`

- length altering characters, i.e.
  - loops: `*` or `+`
  - optional letters: `?`


# Support

- Slicing by individual letters
- Conversion to and from indices (i.e. a-z <--> 0-25)
- Random generation of words

*/

use crate::{letter::Letter, set::Set};
use std::{collections::VecDeque, ops::Deref};

mod parse;
mod random;
mod unparse;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Word(VecDeque<Letter>);

impl Word {
  pub fn push(&mut self, letter: Letter) {
    self.0.push_back(letter)
  }

  pub fn pop(&mut self) -> Option<Letter> {
    self.0.pop_front()
  }

  pub fn pop_back(&mut self) -> Option<Letter> {
    self.0.pop_back()
  }

  pub fn split(mut self) -> Option<(Letter, Self)> {
    let head = self.0.pop_front();
    head.map(|l| (l, self))
  }

  pub(crate) fn letters(&self) -> impl Iterator<Item = &Letter> + '_ {
    self.0.iter()
  }
}

impl Deref for Word {
  type Target = VecDeque<Letter>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromIterator<Letter> for Word {
  fn from_iter<T: IntoIterator<Item = Letter>>(iter: T) -> Self {
    Self(iter.into_iter().collect())
  }
}

impl IntoIterator for Word {
  type Item = String;
  type IntoIter = StringIter;
  fn into_iter(self) -> Self::IntoIter {
    StringIter::new(self)
  }
}

#[derive(Debug, Clone, Default)]
pub struct StringIter {
  /// The word to copy letters from.
  ///
  /// invariant 1: word.iter().all(|letter| !letter.is_empty())
  ///
  /// i.e. no letters in word are empty
  word: Word,
  /// The letters to use for the next string.
  ///
  /// invariant 2: letters.len() == word.len()
  ///
  /// i.e. same number of letters as in word
  letters: Vec<(char, Letter)>,
}

impl StringIter {
  fn new(word: Word) -> StringIter {
    if word.letters().any(|l| l.is_empty()) {
      // we wouldn't return any strings anyway
      StringIter {
        word: Word::default(),
        letters: vec![],
      }
    } else {
      let mut iter = StringIter {
        word,
        letters: vec![],
      };
      iter.fill_from();
      iter
    }
  }

  /// Fills the back of the letters with the remaining letters in the word.
  /// This is used to ensure that the invariant 2 is upheld.
  fn fill_from(&mut self) {
    for mut letter in self.word.range(self.letters.len()..).copied() {
      let c = letter
        .next()
        .expect("invariant 1: no letters in word empty");
      self.letters.push((c, letter));
    }
  }
}

impl Iterator for StringIter {
  type Item = String;
  fn next(&mut self) -> Option<Self::Item> {
    // remove all empty letters from the back
    while self.letters.last()?.1.is_empty() {
      self.letters.pop();
    }
    let (c, letter) = self.letters.last_mut()?;
    *c = letter.next().expect("letter won't be empty");
    self.fill_from();

    Some(self.letters.iter().map(|(c, _)| *c).collect())
  }
}
