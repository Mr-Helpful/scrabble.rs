pub mod random;
pub mod test;

use super::word::{from_index, Word};
use std::{array, io, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trie {
  is_end: bool,
  children: [Option<Box<Trie>>; 26],
}

/*
@note a trie should have some debugging tools:
> Display
*/
use std::fmt::Display;

impl Trie {
  fn branches(&self) -> String {
    let mut child_strs: Vec<String> = self
      .children
      .iter()
      .enumerate()
      .flat_map(|(i, c)| {
        let trie = c.as_ref()?;

        let letter = if trie.is_end {
          format!("╸{}", from_index(i))
        } else {
          format!("╴{}", from_index(i))
        };

        if trie.is_leaf() {
          Some(format!("├─{}\n", letter))
        } else {
          let s = format!("├┬{}\n{}", letter, trie.branches());
          Some(s.replace('\n', "\n│") + "\n")
        }
      })
      .collect();

    // remove prefix characters from the last branch
    if let Some(last) = child_strs.last_mut() {
      *last = last.replacen('├', "└", 1);
      *last = last.replace("\n│", "\n ");
    }

    String::from_iter(child_strs)
      .trim_end()
      .to_owned()
  }

  pub fn len(&self) -> usize {
    let mut l = self.is_end.into();
    for c in self.children.iter() {
      l += c.as_ref().map_or(0, |trie| trie.len())
    }
    l
  }

  pub fn widths(&self) -> Vec<usize> {
    let mut res = vec![];
    for child in self.children.iter() {
      if let Some(trie) = child.as_ref() {
        for (i, count) in trie.widths().into_iter().enumerate() {
          if let Some(current) = res.get_mut(i) {
            *current += count
          } else {
            res.push(count)
          }
        }
      }
    }

    res.insert(0, self.is_end.into());
    res
  }
}

impl Display for Trie {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Trie with {} words:\n{}", self.len(), self.branches())
  }
}

/*
@note a trie is a collection:
> empty, word, is_empty, is_leaf, len, into_iter, from_iter
*/
impl Trie {
  pub fn empty() -> Trie {
    Trie {
      is_end: false,
      children: array::from_fn(|_| None),
    }
  }

  pub fn word(word: Word) -> Self {
    word.split().map_or_else(
      || Trie {
        is_end: true,
        children: array::from_fn(|_| None),
      },
      |(letter, next)| {
        let trie = Box::new(Self::word(next));
        Trie {
          is_end: false,
          children: array::from_fn(|i| letter.has_idx(i).then(|| trie.clone())),
        }
      },
    )
  }

  pub fn str(word: &str) -> Self {
    Self::word(word.try_into().unwrap())
  }

  pub fn all(len: usize) -> Self {
    Self::str(".".repeat(len).as_str())
  }

  pub fn file(path: &Path) -> io::Result<Self> {
    let mut trie = Self::empty();
    trie.load(path)?;
    Ok(trie)
  }

  pub fn is_empty(&self) -> bool {
    !self.is_end
      & self
        .children
        .iter()
        .all(|c| c.as_ref().map_or(true, |n| n.is_empty()))
  }

  pub fn is_leaf(&self) -> bool {
    self.children.iter().all(|c| c.is_none())
  }
}

impl From<()> for Trie {
  fn from(_: ()) -> Self {
    Self::empty()
  }
}

impl From<Word> for Trie {
  fn from(word: Word) -> Self {
    Self::word(word)
  }
}

impl From<&str> for Trie {
  fn from(word: &str) -> Self {
    Self::str(word)
  }
}

impl From<&Path> for Trie {
  fn from(path: &Path) -> Self {
    Self::file(path).ok().unwrap_or_else(Self::empty)
  }
}

impl Iterator for Trie {
  type Item = String;
  fn next(&mut self) -> Option<Self::Item> {
    if self.is_end {
      self.is_end = false;
      return Some(String::from(""));
    }

    self.children.iter_mut().enumerate().find_map(|(i, child)| {
      child.as_mut().and_then(|trie| {
        trie.next().map(|s| format!("{}{}", from_index(i), s))
      })
    })
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

/*
@note a trie may need unused branches pruned to reduce size or clear out
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

  pub fn clear(&mut self) {
    for child in self.children.iter_mut() {
      child.take();
    }
  }
}

/*
@note a trie is a set:
> in, add, remove, union, intersect, difference
*/
use std::ops::{
  Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, DivAssign, Sub, SubAssign,
};

impl Trie {
  fn has_word(&self, word: Word) -> bool {
    match word.split() {
      None => self.is_end,
      Some((letter, word)) => self.children.iter().enumerate().any(|(i, child)| {
        letter.has_idx_unchecked(i)
          && child
            .as_ref()
            .map_or(false, |trie| trie.has_word(word.clone()))
      }),
    }
  }

  pub fn has(&self, word: &str) -> bool {
    self.has_word(word.try_into().unwrap())
  }

  fn has_all_word(&self, word: Word) -> bool {
    match word.split() {
      None => self.is_end,
      Some((letter, word)) => self.children.iter().enumerate().all(|(i, child)| {
        letter.has_idx_unchecked(i)
          && child
            .as_ref()
            .map_or(false, |trie| trie.has_all_word(word.clone()))
      }),
    }
  }

  pub fn has_all(&self, word: &str) -> bool {
    self.has_all_word(word.try_into().unwrap())
  }

  fn add_assign(&mut self, word: &str) {
    self.or_assign(&Trie::str(word))
  }

  fn sub_assign(&mut self, word: &str) {
    self.diff_assign(&Trie::str(word))
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
}

fn immut_op<F: FnMut(&mut Trie)>(trie: &Trie, mut op: F) -> Trie {
  let mut trie = trie.clone();
  op(&mut trie);
  trie
}

impl AddAssign<&str> for Trie {
  fn add_assign(&mut self, rhs: &str) {
    self.add_assign(rhs)
  }
}

impl Add<&str> for &Trie {
  type Output = Trie;
  fn add(self, rhs: &str) -> Self::Output {
    immut_op(self, |trie| *trie += rhs)
  }
}

impl SubAssign<&str> for Trie {
  fn sub_assign(&mut self, rhs: &str) {
    self.sub_assign(rhs)
  }
}

impl Sub<&str> for &Trie {
  type Output = Trie;
  fn sub(self, rhs: &str) -> Self::Output {
    immut_op(self, |trie| *trie -= rhs)
  }
}

impl BitOrAssign<Trie> for Trie {
  fn bitor_assign(&mut self, rhs: Trie) {
    self.or_assign(&rhs)
  }
}

impl BitOr<Trie> for &Trie {
  type Output = Trie;
  fn bitor(self, rhs: Trie) -> Self::Output {
    immut_op(self, |trie| *trie |= rhs.clone())
  }
}

impl BitAndAssign<Trie> for Trie {
  fn bitand_assign(&mut self, rhs: Trie) {
    self.and_assign(&rhs)
  }
}

impl BitAnd<Trie> for &Trie {
  type Output = Trie;
  fn bitand(self, rhs: Trie) -> Self::Output {
    immut_op(self, |trie| *trie &= rhs.clone())
  }
}

impl DivAssign<Trie> for Trie {
  fn div_assign(&mut self, rhs: Trie) {
    self.diff_assign(&rhs)
  }
}

impl Div<Trie> for &Trie {
  type Output = Trie;
  fn div(self, rhs: Trie) -> Self::Output {
    immut_op(self, |trie| *trie /= rhs.clone())
  }
}

/*
@note as tries could get quite large, they should support file operations:
> load_trie, save_trie, load_words, save_words, load, save
 > load and save try to decide the type of operation based on file extension

Trie file format:
a u32 number for each node, traversed in depth-first order
<5 unused bits><1 bit for whether the node is an end><26 bits for each letter>
*/
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::ops::{Shl, Shr};

impl Trie {
  fn load_trie(&mut self, file: &mut File) -> io::Result<()> {
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    let node = u32::from_be_bytes(buf);
    if node & (1 << 26) > 0 {
      self.is_end = true
    }

    for (i, c) in self.children.iter_mut().enumerate() {
      if node & (1 << i) > 0 {
        if c.is_none() {
          *c = Some(Box::new(Self::empty()))
        }
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
      if c.is_some() {
        node |= 1 << i
      }
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

  fn save_words(self, file: &mut File) -> io::Result<()> {
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
        "File type not supported, please load from either a .tre or .txt file.",
      )),
    }
  }

  fn save(self, path: &Path) -> io::Result<()> {
    let mut file = File::create(path)?;
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
      Some("tre") => self.save_trie(&mut file),
      Some("txt") => self.save_words(&mut file),
      Some(_) | None => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "File type not supported, please save to either a .tre or .txt file.",
      )),
    }
  }
}

impl Shl<&Path> for &Trie {
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
    self.clone().save(rhs)?;
    Ok(self)
  }
}
