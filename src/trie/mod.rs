use crate::word::Word;
use std::{
  ops::{BitAnd, BitOr, Deref, DerefMut, Sub},
  str::FromStr,
};

mod node_trait;
pub use node_trait::TrieNode;
use nom::error::Error;

pub struct Trie<N: TrieNode>(N);

impl<N: TrieNode> Default for Trie<N> {
  fn default() -> Self {
    Self(N::empty())
  }
}
impl<N: TrieNode> From<Word> for Trie<N> {
  fn from(value: Word) -> Self {
    Self(N::from_word(value))
  }
}
impl<N: TrieNode> FromStr for Trie<N> {
  type Err = Error<String>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(N::from_word(s.parse::<Word>()?)))
  }
}

impl<N: TrieNode> Deref for Trie<N> {
  type Target = N;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl<N: TrieNode> DerefMut for Trie<N> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<N: TrieNode> BitAnd<&Trie<N>> for &Trie<N> {
  type Output = Trie<N>;
  fn bitand(self, rhs: &Trie<N>) -> Self::Output {
    Trie(self.and(rhs.deref()))
  }
}
impl<N: TrieNode, I: Into<Word> + Clone> BitAnd<&I> for &Trie<N> {
  type Output = Trie<N>;
  fn bitand(self, rhs: &I) -> Self::Output {
    let word: Word = rhs.clone().into();
    let trie: Trie<N> = word.into();
    self & &trie
  }
}

impl<N: TrieNode> BitOr<&Trie<N>> for &Trie<N> {
  type Output = Trie<N>;
  fn bitor(self, rhs: &Trie<N>) -> Self::Output {
    Trie(self.or(rhs.deref()))
  }
}
impl<N: TrieNode, I: Into<Word> + Clone> BitOr<&I> for &Trie<N> {
  type Output = Trie<N>;
  fn bitor(self, rhs: &I) -> Self::Output {
    let word: Word = rhs.clone().into();
    let trie: Trie<N> = word.into();
    self | &trie
  }
}

impl<N: TrieNode> Sub<&Trie<N>> for &Trie<N> {
  type Output = Trie<N>;
  fn sub(self, rhs: &Trie<N>) -> Self::Output {
    Trie(self.diff(rhs.deref()))
  }
}
impl<N: TrieNode, I: Into<Word> + Clone> Sub<&I> for &Trie<N> {
  type Output = Trie<N>;
  fn sub(self, rhs: &I) -> Self::Output {
    let word: Word = rhs.clone().into();
    let trie: Trie<N> = word.into();
    self - &trie
  }
}
