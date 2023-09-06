use crate::set::Set;

pub mod parse;
mod random;
pub mod unparse;

pub fn into_index(c: char) -> usize {
  (c as usize) - ('a' as usize)
}

pub fn from_index(i: usize) -> char {
  (i + ('a' as usize)) as u8 as char
}

// todo: convert to a bitset for performance
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Letter([bool; 26]);

impl Letter {
  pub fn has_idx(&self, i: usize) -> bool {
    (0..26).contains(&i) && self.0[i]
  }

  pub(crate) fn has_idx_unchecked(&self, i: usize) -> bool {
    self.0[i]
  }

  pub(crate) fn peek_idx(&self) -> Option<usize> {
    self.0.iter().position(|&b| b)
  }

  pub(crate) fn indices(&self) -> impl Iterator<Item = usize> + '_ {
    self
      .0
      .iter()
      .enumerate()
      .filter_map(|(i, &b)| b.then_some(i))
  }

  pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
    self.indices().map(from_index)
  }
}

impl Iterator for Letter {
  type Item = char;
  fn next(&mut self) -> Option<Self::Item> {
    let idx = self.peek_idx()?;
    self.0[idx] = false;
    Some(from_index(idx))
  }
}

impl Extend<char> for Letter {
  fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
    for c in iter {
      self.0[into_index(c)] = true;
    }
  }
}

impl FromIterator<char> for Letter {
  fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
    let mut letter = Self::default();
    letter.extend(iter);
    letter
  }
}

impl Set for Letter {
  fn singleton(item: Self::Item) -> Self {
    let mut letter = Self::default();
    letter.0[into_index(item)] = true;
    letter
  }

  fn len(&self) -> usize {
    self.indices().count()
  }
  fn is_empty(&self) -> bool {
    self.0 == [false; 26]
  }
  fn contains(&self, item: &Self::Item) -> bool {
    self.has_idx(into_index(*item))
  }
  fn subset(&self, other: &Self) -> bool {
    for i in 0..26 {
      if self.0[i] && !other.0[i] {
        return false;
      }
    }
    true
  }

  fn insert(&mut self, item: Self::Item) -> bool {
    let idx = into_index(item);
    let prev = self.0[idx];
    self.0[idx] = true;
    prev
  }
  fn retain<F>(&mut self, mut f: F)
  where
    F: FnMut(&Self::Item) -> bool,
  {
    for i in 0..26 {
      if !f(&from_index(i)) {
        self.0[i] = false;
      }
    }
  }
  fn delete(&mut self, item: &Self::Item) -> bool {
    let idx = into_index(*item);
    let prev = self.0[idx];
    self.0[idx] = false;
    prev
  }
  fn clear(&mut self) {
    self.0 = [false; 26];
  }

  fn intersect(&mut self, other: &Self) {
    for i in 0..26 {
      if !other.0[i] {
        self.0[i] = false;
      }
    }
  }
  fn remove(&mut self, other: &Self) {
    for i in 0..26 {
      if other.0[i] {
        self.0[i] = false;
      }
    }
  }
}
