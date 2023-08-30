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

use std::{collections::VecDeque, str::FromStr};

pub mod random;

pub fn into_index(c: char) -> usize {
  (c as usize) - ('a' as usize)
}

pub fn from_index(i: usize) -> char {
  (i + ('a' as usize)) as u8 as char
}

#[derive(Debug, Clone)]
pub struct Letter([bool; 26]);

impl Letter {
  fn all() -> Self {
    Self([true; 26])
  }

  pub fn has(&self, c: char) -> bool {
    self.has_idx(into_index(c))
  }

  pub fn has_unchecked(&self, c: char) -> bool {
    self.has_idx_unchecked(into_index(c))
  }

  pub fn has_idx(&self, i: usize) -> bool {
    (0..26).contains(&i) && self.0[i]
  }

  pub fn has_idx_unchecked(&self, i: usize) -> bool {
    self.0[i]
  }
}

impl TryFrom<char> for Letter {
  type Error = String;
  fn try_from(value: char) -> Result<Self, Self::Error> {
    match value {
      '.' => Ok(Self::all()),
      c @ 'a'..='z' => Ok({
        let mut mask = [false; 26];
        mask[into_index(c)] = true;
        Letter(mask)
      }),
      c => Err(format!("Unknown char `{c}`")),
    }
  }
}

impl TryInto<char> for Letter {
  type Error = String;
  fn try_into(self) -> Result<char, Self::Error> {
    let mut all = true;
    let mut last = Err("Letter matching nothing cannot be summarised");

    for i in 0..26 {
      if self.0[i] {
        last = match last {
          Ok(_) => Err("Letter matching 1..26 characters cannot be summarised"),
          Err("Letter matching nothing cannot be summarised") => Ok(i),
          e => e,
        }
      } else {
        all = false
      }
    }

    if all {
      Ok('.')
    } else {
      last.map(from_index).map_err(String::from)
    }
  }
}

impl FromStr for Letter {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut start = None; // start will always be alphabetic
    let mut prior = Some('a');
    let mut char_set = [false; 26];

    for c in s.chars() {
      match c {
        '-' => {
          if prior == Some('-') {
            return Err("Double dash for range `--`".into());
          }
          start = prior;
          prior = Some('-')
        }
        c @ 'a'..='z' => {
          let s = start.unwrap_or(c);
          for b in &mut char_set[into_index(s)..=into_index(c)] {
            *b = true
          }
          start = None;
          prior = Some(c)
        }
        '[' => return Err("Nested open bracket `[...[`".into()),
        c => return Err(format!("Unknown char `{c}`")),
      }
    }

    if let Some(s) = start {
      for b in &mut char_set[into_index(s)..=into_index('z')] {
        *b = true
      }
    }

    Ok(Letter(char_set))
  }
}

impl From<Letter> for String {
  fn from(val: Letter) -> Self {
    let mut ranges = vec![];

    for i in (0..26).filter(|&i| val.0[i]) {
      let (s, e) = ranges.pop().unwrap_or((i, i));
      if e + 1 >= i {
        ranges.push((s, i))
      } else {
        ranges.push((s, e));
        ranges.push((i, i))
      }
    }

    if ranges.len() == 1 {
      let (s, e) = ranges.pop().unwrap();
      if s == e {
        return from_index(s).into();
      }
      if (s == 0) & (e == 25) {
        return ".".into();
      }
    }

    format!(
      "[{}]",
      ranges
        .into_iter()
        .map(|(s, e)| {
          if s == 0 {
            format!("-{}", from_index(e))
          } else if e == 25 {
            format!("{}-", from_index(s))
          } else if s == e {
            s.to_string()
          } else {
            format!("{}-{}", from_index(s), from_index(e))
          }
        })
        .collect::<String>()
    )
  }
}

impl IntoIterator for Letter {
  type Item = char;
  type IntoIter = LetterIter;
  fn into_iter(self) -> Self::IntoIter {
    LetterIter {
      set: self.0,
      idx: 0,
    }
  }
}

pub struct LetterIter {
  set: [bool; 26],
  idx: usize,
}

impl Iterator for LetterIter {
  type Item = char;
  fn next(&mut self) -> Option<Self::Item> {
    while self.idx < 26 {
      if self.set[self.idx] {
        return Some(from_index(self.idx));
      }
      self.idx += 1
    }
    None
  }
}

#[derive(Debug, Clone)]
pub struct Word(VecDeque<Letter>);

impl FromStr for Word {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut letters = VecDeque::new();
    let mut group: Option<String> = None;

    for c in s.chars() {
      match c {
        ']' => letters.push_back(
          group
            .take()
            .ok_or_else(|| "Unmatched closing bracket `...]`".to_owned())?
            .parse()?,
        ),
        c @ ('a'..='z' | '-') if group.is_some() => group = group.map(|s| format!("{s}{c}")),
        c @ ('a'..='z' | '.') => letters.push_back(c.try_into()?),
        '[' => group = Some(String::from("")),
        c => return Err(format!("Unknown char `{c}`")),
      }
    }

    Ok(Word(letters))
  }
}

impl From<Word> for String {
  fn from(val: Word) -> Self {
    val.map(|letter| -> String { letter.into() }).collect()
  }
}

impl TryFrom<&str> for Word {
  type Error = String;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    String::from(value).parse()
  }
}

impl Word {
  pub fn push(&mut self, letter: Letter) {
    self.0.push_back(letter)
  }

  // pub fn push_char(&mut self, ch: char) {
  //   self.0.push_back(ch.into())
  // }

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
}

impl Iterator for Word {
  type Item = Letter;
  fn next(&mut self) -> Option<Self::Item> {
    self.pop()
  }
}
