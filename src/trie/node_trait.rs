use crate::word::Word;
use std::{
  iter::{Flatten, Scan},
  ops::RangeInclusive,
};

type DepthFirstGenerator<T, R> =
  for<'a, 'b> fn(&'a mut R, (&'b T, Option<char>)) -> Option<Option<R>>;
type DepthFirstScan<'a, T, R> = Scan<DepthFirstIterator<'a, T>, R, DepthFirstGenerator<T, R>>;

pub trait TrieNode: Clone + Sized {
  /*--------------------------------------------------*/
  /*-             @note Required methods             -*/
  /*--------------------------------------------------*/
  /// Creates a completely empty trie
  fn empty() -> Self;

  /// Whether a node represents the end of a word
  fn is_end(&self) -> bool;
  /// Setting whether a node represents the end of the word
  fn set_end(&mut self, end: bool);
  /// Whether a node has any children
  fn is_leaf(&self) -> bool;

  /// Fetches a immutable reference to a child for a given character
  fn get_child(&self, c: char) -> Option<&Self>;
  /// Fetches a mutable reference to a child for a given character
  fn get_mut_child(&mut self, c: char) -> Option<&mut Self>;
  /// Set the child for a given character
  fn set_child(&mut self, c: char, other: Option<Self>);

  /*--------------------------------------------------*/
  /*-             @note Optional methods             -*/
  /*--------------------------------------------------*/
  /// Generates a trie from a provided word
  fn from_word(mut word: Word) -> Self {
    let mut node = Self::empty();

    if let Some(letter) = word.next() {
      let sub = Self::from_word(word);
      for c in letter {
        node.set_child(c, Some(sub.clone()))
      }
    } else {
      node.set_end(true)
    }

    node
  }

  /// Whether a node has no words in it
  fn is_empty(&self) -> bool {
    !self.is_end()
      && ('a'..='z')
        .flat_map(|c| self.get_child(c))
        .all(|c| c.is_empty())
  }
  /// Whether any of the words in other occur in self
  fn has_any(&self, other: &Self) -> bool {
    self.is_end()
      || other.chars().into_iter().any(|c| {
        self
          .get_child(c)
          .zip(other.get_child(c))
          .map_or(false, |(sub0, sub1)| sub0.has_any(sub1))
      })
  }
  /// Whether all of the words in other occur in self
  fn has_all(&self, other: &Self) -> bool {
    self.is_end()
      && other.chars().into_iter().all(|c| {
        self
          .get_child(c)
          .zip(other.get_child(c))
          .map_or(false, |(sub0, sub1)| sub0.has_all(sub1))
      })
  }

  /// A naive implementation that gets all the characters in a node
  fn chars(&self) -> Vec<char> {
    ('a'..='z')
      .filter(|&c| self.get_child(c).is_some())
      .collect()
  }
  /// A naive implementation that gets children of a node
  fn children(&self) -> Vec<&Self> {
    ('a'..='z').filter_map(|c| self.get_child(c)).collect()
  }
  /// Gets a reference to a child, filling it with an empty trie
  /// if there isn't a child for that character
  fn get_or_insert(&mut self, c: char) -> &Self {
    if self.get_child(c).is_none() {
      self.set_child(c, Some(Self::empty()))
    }
    self.get_child(c).expect("child should be filled")
  }
  /// Gets a mutable reference to a child, filling it with an empty trie
  /// if there isn't a child for that character
  fn get_mut_or_insert(&mut self, c: char) -> &mut Self {
    if self.get_child(c).is_none() {
      self.set_child(c, Some(Self::empty()))
    }
    self.get_mut_child(c).expect("child should be filled")
  }

  /// The intersection of two trie nodes.
  ///
  /// A word will only be included if it is present in both tries.
  fn and(&self, other: &Self) -> Self {
    let mut tree = self.clone();
    tree.set_end(self.is_end() & other.is_end());

    for c in 'a'..='z' {
      let sub = tree
        .get_child(c)
        .zip(other.get_child(c))
        .map(|(sub0, sub1)| sub0.and(sub1));
      tree.set_child(c, sub)
    }

    tree
  }
  /// The union of two trie nodes.
  ///
  /// A word will be included if it present in either trie.
  fn or(&self, other: &Self) -> Self {
    let mut tree = Self::empty();
    tree.set_end(self.is_end() | other.is_end());

    for c in 'a'..='z' {
      let sub = match (self.get_child(c), other.get_child(c)) {
        (Some(sub0), Some(sub1)) => Some(sub0.or(sub1)),
        (Some(sub0), None) => Some(sub0.or(&Self::empty())),
        (None, Some(sub1)) => Some(sub1.or(&Self::empty())),
        (None, None) => None,
      };
      tree.set_child(c, sub)
    }

    tree
  }
  /// The asymmetric difference of two tries.
  ///
  /// A word will be present if it's in the first but not second trie.
  ///
  fn diff(&self, other: &Self) -> Self {
    let mut tree = Self::empty();
    tree.set_end(tree.is_end() & !other.is_end());

    for c in 'a'..='z' {
      let sub = match (self.get_child(c), other.get_child(c)) {
        (Some(sub0), Some(sub1)) => Some(sub0.diff(sub1)),
        (Some(sub0), None) => Some(sub0.clone()),
        (None, _) => None,
      };
      tree.set_child(c, sub)
    }

    tree
  }

  fn dfs(&self) -> DepthFirstIterator<'_, Self> {
    DepthFirstIterator {
      stack: vec![(self, 'a'..='z')],
    }
  }

  // fn words(&self) -> Flatten<DepthFirstScan<'_, Self, Word>> {
  //   fn scan_to_word<T: TrieNode>(
  //     word: &mut Word,
  //     (node, movement): (&T, Option<char>),
  //   ) -> Option<Option<Word>> {
  //     match movement {
  //       Some(c) => word.push_char(c),
  //       None => {
  //         word.pop();
  //       }
  //     };
  //     movement.map(|_| node.is_end().then(|| word.clone()))
  //   }

  //   self
  //     .dfs()
  //     .scan(
  //       "".to_owned(),
  //       scan_to_word
  //         as for<'a, 'b> fn(&'a mut Word, (&'b Self, Option<char>)) -> Option<Option<Word>>,
  //     )
  //     .flatten()
  // }

  /// Generates an iterator over all the strings stored in the node
  ///
  /// I'm sorry the type for this is just awful to write, but I couldn't use
  /// `impl Iterator<Item = String>` as that's not supported by rust yet, due
  /// to Higher Kinded Type (HKT) issues.
  fn strings(&self) -> Flatten<DepthFirstScan<'_, Self, String>> {
    fn scan_to_string<T: TrieNode>(
      string: &mut String,
      (node, movement): (&T, Option<char>),
    ) -> Option<Option<String>> {
      match movement {
        Some(c) => string.push(c),
        None => {
          string.pop();
        }
      };
      movement.map(|_| node.is_end().then(|| string.clone()))
    }

    self
      .dfs()
      .scan(
        "".to_owned(),
        scan_to_string
          as for<'a, 'b> fn(&'a mut String, (&'b Self, Option<char>)) -> Option<Option<String>>,
      )
      .flatten()
  }

  /// Generates an iterator over references to the tries that occur at the end
  /// of each word in other, or an error if the trie doesn't extend that far.
  fn extract<'a>(&'a self, other: &'a Self) -> ExtractIterator<'_, Self> {
    ExtractIterator {
      stack: vec![(self, other, 'a'..='z')],
    }
  }
}

#[derive(Clone)]
pub struct DepthFirstIterator<'a, T: TrieNode> {
  stack: Vec<(&'a T, RangeInclusive<char>)>,
}

impl<'a, T: TrieNode> Iterator for DepthFirstIterator<'a, T> {
  type Item = (&'a T, Option<char>);
  fn next(&mut self) -> Option<Self::Item> {
    let (node, left) = self.stack.last_mut()?;
    let entry = left.next().and_then(|c| {
      let sub = node.get_child(c)?;
      Some((sub, c))
    });

    if let Some((sub, c)) = entry {
      self.stack.push((sub, 'a'..='z'));
      Some((sub, Some(c)))
    } else {
      let (node, _) = self.stack.pop()?;
      Some((node, None))
    }
  }
}

#[derive(Clone)]
pub struct StringIterator<'a, T: TrieNode> {
  stack: Vec<(&'a T, RangeInclusive<char>)>,
  prefix: String,
}

impl<'a, T: TrieNode> Iterator for StringIterator<'a, T> {
  type Item = String;
  fn next(&mut self) -> Option<Self::Item> {
    let (node, left) = self.stack.last_mut()?;
    if node.is_end() {
      return Some(self.prefix.clone());
    }

    let entry = left.next().and_then(|c| {
      let sub = node.get_child(c)?;
      Some((c, sub))
    });
    if let Some((c, sub)) = entry {
      self.prefix.push(c);
      self.stack.push((sub, 'a'..='z'));
    } else {
      self.prefix.pop();
      self.stack.pop();
    }

    self.next()
  }
}

#[derive(Clone)]
pub struct ExtractIterator<'a, T: TrieNode> {
  stack: Vec<(&'a T, &'a T, RangeInclusive<char>)>,
}

impl<'a, T: TrieNode> Iterator for ExtractIterator<'a, T> {
  type Item = Result<&'a T, String>;
  fn next(&mut self) -> Option<Self::Item> {
    let (tree0, tree1, mut left) = self.stack.pop()?;
    if tree0.is_end() & tree1.is_end() {
      return Some(Ok(tree0));
    }
    if left.is_empty() {
      return self.next();
    }

    let c = left.next().unwrap();
    self.stack.push((tree0, tree1, left));
    let optn = tree0.get_child(c).zip(tree1.get_child(c));
    if let Some((sub0, sub1)) = optn {
      self.stack.push((sub0, sub1, 'a'..='z'));
      self.next()
    } else {
      Some(Err(format!("Cannot find child for `{c}`")))
    }
  }
}
