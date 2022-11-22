use crate::trie_paths::{
  split_path, char_set_from_group, into_index, from_index
};

use std::{
  array,
  ops::{
    Add, AddAssign, BitOrAssign, BitOr, BitAnd, BitAndAssign, DivAssign, Div, Shl, Shr, Sub, SubAssign
  },
  fs::File,
  io::{self, BufReader, Read, Write, BufRead},
  path::Path
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trie {
  is_end: bool,
  children: [Option<Box<Trie>>; 26]
}

/*
- a trie is a collection:
> empty, word, path, is_empty, into_iter, from_iter
*/
impl Trie {
  pub fn empty() -> Trie {
    Trie {
      is_end: false,
      children: array::from_fn(|_| None)
    }
  }

  pub fn word(word: &str) -> Trie {
    let (head, tail) = split_path(word);
    let char_set = char_set_from_group(head);
    Trie {
      is_end: word == "",
      children: array::from_fn(|i| 
        if char_set & (1 << i) == 0 { None }
        else { Some(Box::new(Self::word(tail))) }
      )
    }
  }

  pub fn file(path: &Path) -> io::Result<Trie> {
    let mut trie = Self::empty();
    trie.load(path)?;
    Ok(trie)
  }

  pub fn is_empty(&self) -> bool {
    !self.is_end & self.children.iter().all(
      |c| c.as_ref().map_or(true, |n| n.is_empty())
    )
  }

  pub fn is_leaf(&self) -> bool {
    self.children.iter().all(|c| c.is_none())
  }
}

pub struct TrieIter(String, Vec<Trie>);

impl<'a> Iterator for TrieIter {
  type Item = String;
  fn next(&mut self) -> Option<Self::Item> {
    let mut trie = self.1.pop()?;
    if trie.is_end {
      trie.is_end = false;
      self.1.push(trie);
      Some(self.0.clone())
    } else {
      for (u, child) in trie.children.iter_mut().enumerate() {
        if let Some(node) = child.take() {
          self.0.push(from_index(u));
          self.1.push(trie);
          self.1.push(*node);
          return self.next()
        }
      }

      let last = self.0.len();
      self.0.truncate(last - 1);
      self.next()
    }
  }
}

impl<'a> IntoIterator for &'a Trie {
  type Item = String;
  type IntoIter = TrieIter;
  fn into_iter(self) -> Self::IntoIter {
    TrieIter(String::from(""), vec![self.clone()])
  }
}

impl FromIterator<String> for Trie {
  fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
    let mut trie = Trie::empty();
    for s in iter {
      trie += &s
    }
    trie
  }
}

#[cfg(test)]
mod collection_tests {
  use super::*;

  #[test]
  fn empty_constructor_leaf() {
    let trie = Trie::empty();
    assert!(trie.is_leaf())
  }

  #[test]
  fn empty_constructor_empty() {
    let trie = Trie::empty();
    assert!(trie.is_empty())
  }

  #[test]
  fn word_constructor_empty() {
    let trie = Trie::word("");
    assert!(trie.is_end);
    assert!(trie.is_leaf())
  }

  #[test]
  fn word_constructor_char() {
    let trie = Trie::word("a");
    let mut i = trie.into_iter();
    assert_eq!(i.next(), Some(String::from("a")));
    assert_eq!(i.next(), None)
  }
}

/*
- a trie may need unused branches pruned to reduce size or clear out
> prune, clear
*/
impl Trie {
  pub fn prune(&mut self) {
    for child in self.children.iter_mut() {
      if let Some(node) = child {
        node.prune();
        if node.is_empty() {
          *child = None
        }
      }
    }
  }

  pub fn clone(&mut self) {
    for child in self.children.iter_mut() {
      child.take();
    }
  }
}

/*
a trie is a set:
> in, add, remove, union, intersect, difference
*/
impl Trie {
  fn immut_op<F: FnMut(&mut Trie)>(trie: Trie, mut op: F) -> Trie {
    let mut cloned = trie.clone();
    op(&mut cloned);
    cloned
  }

  // @todo make this work on the extended format of strings
  pub fn has(&self, word: &str) -> bool {
    if word == "" { return self.is_end }
    let (head, tail) = word.split_at(1);
    let c = head.chars().next().unwrap();
    let i = into_index(c);

    self.children.get(i)
      .map_or(false, |child| child.as_ref()
        .map_or(false, |trie| trie.has(tail))
      )
  }

  // @todo make this work on the extended format of strings
  fn add_assign(&mut self, word: &str) {
    if word == "" { self.is_end = true; return }

    let (head, tail) = word.split_at(1);
    let c = head.chars().next().unwrap();
    let i = into_index(c);

    if self.children[i].is_none() {
      let child = Box::new(Self::empty());
      self.children[i] = Some(child);
    }

    self.children[i].as_mut().map(|n| n.add_assign(tail));
  }

  fn add(self, word: &str) -> Trie {
    Self::immut_op(self, |t| t.add_assign(word))
  }

  // @todo make this work on the extended format of strings
  fn sub_assign(&mut self, word: &str) {
    if word == "" { self.is_end = false; return }

    let (head, tail) = word.split_at(1);
    let c = head.chars().next().unwrap();
    let i = into_index(c);

    self.children[i].as_mut().map(|n| n.sub_assign(tail));
  }

  fn sub(self, word: &str) -> Trie {
    Self::immut_op(self, |t| t.sub_assign(word))
  }

  fn or_assign(&mut self, trie: &Trie) {
    self.is_end |= trie.is_end;

    for (selfc, triec) in self.children.iter_mut().zip(trie.children.iter()) {
      if let Some(trien) = triec {
        if let Some(selfn) = selfc {
          selfn.or_assign(trien)
        } else {
          *selfc = Some(trien.clone())
        }
      }
    }
  }

  fn or(self, trie: &Trie) -> Trie {
    Self::immut_op(self, |t| t.or_assign(trie))
  }

  fn and_assign(&mut self, trie: &Trie) {
    self.is_end &= trie.is_end;

    for (selfc, triec) in self.children.iter_mut().zip(trie.children.iter()) {
      if let Some(trien) = triec {
        if let Some(selfn) = selfc {
          selfn.and_assign(trien)
        }
      } else {
        *selfc = None
      }
    }
  }

  fn and(self, trie: &Trie) -> Trie {
    Self::immut_op(self, |t| t.and_assign(trie))
  }

  fn diff_assign(&mut self, trie: &Trie) {
    self.is_end &= !trie.is_end;

    for (selfc, triec) in self.children.iter_mut().zip(trie.children.iter()) {
      if let Some(trien) = triec {
        if let Some(selfn) = selfc {
          selfn.diff_assign(trien)
        }
      }
    }
  }

  fn diff(self, trie: &Trie) -> Trie {
    Self::immut_op(self, |t| t.diff_assign(trie))
  }
}

impl AddAssign<&str> for Trie {
  fn add_assign(&mut self, rhs: &str) { self.add_assign(rhs) }
}

impl Add<&str> for Trie {
  type Output = Trie;
  fn add(self, rhs: &str) -> Self::Output { self.add(rhs) }
}

impl SubAssign<&str> for Trie {
  fn sub_assign(&mut self, rhs: &str) { self.sub_assign(rhs) }
}

impl Sub<&str> for Trie {
  type Output = Trie;
  fn sub(self, rhs: &str) -> Self::Output { self.sub(rhs) }
}

impl BitOrAssign<Trie> for Trie {
  fn bitor_assign(&mut self, rhs: Trie) { self.or_assign(&rhs) }
}

impl BitOr<Trie> for Trie {
  type Output = Trie;
  fn bitor(self, rhs: Trie) -> Self::Output { self.or(&rhs) }
}

impl BitAndAssign<Trie> for Trie {
  fn bitand_assign(&mut self, rhs: Trie) { self.and_assign(&rhs) }
}

impl BitAnd<Trie> for Trie {
  type Output = Trie;
  fn bitand(self, rhs: Trie) -> Self::Output { self.and(&rhs) }
}

impl DivAssign<Trie> for Trie {
  fn div_assign(&mut self, rhs: Trie) { self.diff_assign(&rhs) }
}

impl Div<Trie> for Trie {
  type Output = Trie;
  fn div(self, rhs: Trie) -> Self::Output { self.diff(&rhs) }
}

/*
as tries are liable to get quite large, they should support file operations:
> load_trie, save_trie, load_words, save_words, load, save
 > load and save try to decide the type of operation based on file extension

Trie file format:
a u32 number for each node, traversed in depth-first order
<5 unused bits><1 bit for whether the node is an end><26 bits for each letter>
*/
impl Trie {
  fn load_trie(&mut self, file: &mut File) -> io::Result<()> {
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    let node = u32::from_be_bytes(buf);
    if node & (1 << 26) > 0 { self.is_end = true }

    for (i, c) in self.children.iter_mut().enumerate() {
      if node & (1 << i) > 0 {
        if c.is_none() { *c = Some(Box::new(Self::empty())) }
        let trie = c.as_mut().unwrap();
        trie.load_trie(file)?;
        *c = Some(trie.to_owned())
      }
    }

    Ok(())
  }

  fn save_trie(&self, file: &mut File) -> io::Result<()> {
    let mut node: u32 = if self.is_end { 1 << 26 } else { 0 };
    for (i, c) in self.children.iter().enumerate() {
      if c.is_some() { node |= 1 << i }
    }

    file.write_all(node.to_be_bytes().as_ref())?;

    for child in self.children.iter() {
      if let Some(trie) = child.as_ref() {
        trie.save_trie(file)?
      }
    }

    Ok(())
  }

  fn load_words(&mut self, file: &mut File) -> io::Result<()> {
    for line in BufReader::new(file).lines() {
      self.add_assign(&line?);
    }

    Ok(())
  }

  fn save_words(&self, file: &mut File) -> io::Result<()> {
    for word in self {
      writeln!(file, "{}", word)?
    }

    Ok(())
  }

  fn load(&mut self, path: &Path) -> io::Result<()> {
    let mut file = File::open(path)?;
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
      Some("tre") => self.load_trie(&mut file),
      Some("txt") => self.load_words(&mut file),
      Some(_) | None => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "File type not supported, please load from either a .tre or .txt file."
      ))
    }
  }

  fn save(&self, path: &Path) -> io::Result<()> {
    let mut file = File::create(path)?;
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
      Some("tre") => self.save_trie(&mut file),
      Some("txt") => self.save_words(&mut file),
      Some(_) | None => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "File type not supported, please save to either a .tre or .txt file."
      ))
    }
  }
}

impl Shl<&Path> for Trie {
  type Output = io::Result<Trie>;
  fn shl(self, rhs: &Path) -> Self::Output {
    let mut trie = self.clone();
    trie.load(rhs)?;
    Ok(trie)
  }
}

impl Shr<&Path> for Trie {
  type Output = io::Result<Trie>;
  fn shr(self, rhs: &Path) -> Self::Output {
    self.save(rhs)?;
    Ok(self)
  }
}
