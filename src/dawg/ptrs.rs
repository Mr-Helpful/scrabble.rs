use std::{
  array,
  borrow::Borrow,
  ops::{Deref, DerefMut},
  rc::{Rc, Weak},
};

use super::word::Word;
use weak_table::{traits::WeakElement, PtrWeakHashSet};

/** A single node in a Directed Acyclic Word Graph (DAWG)

# Structure

`Dawg::end_node: Weak<Dawg>`:<br/>
Every node in the Dawg needs to share the same end node, in order to make recursive merging of nodes possible. Hence each node needs a reference to this common ending node, even if it's not using it.

`Dawg::children: [Option<Rc<Dawg>>; 26]`:<br/>
We need to reserve a child for each letter and a child for the ending node. Nodes won't necessarily have a node for each letter, hence the `Option` type.

`Dawg::parents: Vec<Weak<Dawg>>`:<br\>
Whilst merging, we want to avoid checking every single node to see if it can be merged (i.e. points to all the same nodes as `self`). By storing parents for each node we can at least find another node with one pointer in common (by following the path `self.child.parent`).
*/
type RcLink = Rc<Node>;

#[derive(Clone)]
pub struct Dawg(RcLink);
impl Dawg {
  pub fn downgrade(&self) -> WeakDawg {
    WeakDawg(Rc::downgrade(&self.0))
  }
}

type WeakLink = Weak<Node>;

#[derive(Clone)]
pub struct WeakDawg(WeakLink);
impl WeakDawg {
  fn upgrade(&self) -> Option<Dawg> {
    self.0.upgrade().map(Dawg)
  }
}

/* Implementations for WeakHashSet */
impl WeakElement for WeakDawg {
  type Strong = Dawg;
  fn new(view: &Self::Strong) -> Self {
    WeakDawg(Rc::downgrade(&view.0))
  }
  fn view(&self) -> Option<Self::Strong> {
    self.0.upgrade().map(Dawg)
  }
}

type Children = [Option<Dawg>; 27];
type Parents = PtrWeakHashSet<WeakDawg>;

#[derive(Clone)]
pub struct Node {
  end_node: WeakDawg,
  children: Children,
  parents: Parents,
}

/* Derefencing */
impl Dawg {
  pub fn borrow(&self) -> impl Deref<Target = Node> + '_ {
    self.0.as_ref().borrow()
  }
}
impl Deref for Dawg {
  type Target = Node;
  fn deref(&self) -> &Self::Target {
    self.0.as_ref().borrow()
  }
}
impl DerefMut for Dawg {
  fn deref_mut(&mut self) -> &mut Self::Target {
    Rc::get_mut(&mut self.0).unwrap()
  }
}

/* Dawg equality */
impl Dawg {
  fn eq_with<F: Fn(&Dawg, &Dawg) -> bool>(&self, other: &Dawg, f: F) -> bool {
    self
      .children
      .iter()
      .zip(other.children.iter())
      .all(|(c1, c2)| match (c1, c2) {
        (Some(l1), Some(l2)) => f(l1, l2),
        (None, None) => true,
        _ => false,
      })
  }

  /// Determines whether one dawg is equal to another,
  /// down to pointer equality on the node's children.
  pub fn ptr_eq(&self, other: &Self) -> bool {
    self.eq_with(other, |d1, d2| Rc::ptr_eq(&d1.0, &d2.0))
  }
}
impl PartialEq for Dawg {
  fn eq(&self, other: &Self) -> bool {
    self.eq_with(other, |d1, d2| d1.eq(d2))
  }
}

impl Dawg {
  /// Attempts to find an equivalent node to this one within the Dawg
  /// For this we can rely on the fact that if two nodes share a child
  /// then we can reach one node from another via the common child
  pub fn find_eq(&self) -> Option<Dawg> {
    let child = self.children.iter().find_map(|x| x.as_ref())?;
    let mut siblings = child.parents.iter();
    siblings.find_map(|parent| Some(parent).filter(|node| self.ptr_eq(node)))
  }

  /// Attempts to compact the representation of a Dawg node
  /// using pre-existing duplicate nodes within the Dawg
  /// This is what keeps DAWG representations distinct from tries
  pub fn merge(&mut self) {
    for child in self.children.iter_mut().flatten() {
      if let Some(node) = child.find_eq() {
        *child = node
      }
    }
  }
}

/*
@note a dawg should have some debugging tools:
> Display
*/

/*
@note a trie is a collection:
> empty, word, path, is_empty, is_leaf, len, into_iter, from_iter
*/
impl Dawg {
  /// There's quite a bit of boilerplate with constructing a node
  /// So we create a factory function for a Dawg
  fn from_args(end_node: WeakDawg, children: Children) -> Self {
    Dawg(Rc::new(Node {
      end_node,
      children,
      parents: Parents::new(),
    }))
  }

  fn end() -> WeakDawg {
    WeakDawg(Rc::downgrade(&Rc::new_cyclic(|node| Node {
      end_node: WeakDawg(node.clone()),
      children: array::from_fn(|_| None),
      parents: Parents::new(),
    })))
  }

  pub fn empty() -> Self {
    Self::from_args(Self::end(), array::from_fn(|_| None))
  }

  fn add_end(mut self) -> Self {
    let cloned = self.clone();
    self.children[26] = self.end_node.upgrade();
    self.children[26]
      .as_mut()
      .map(|node| node.parents.insert(cloned));
    self
  }

  pub fn word(word: Word) -> Self {
    word.split().map_or_else(
      || Self::empty().add_end(),
      |(letter, next)| {
        let mut node = Self::word(next);
        let dawg = Self::from_args(
          node.end_node.clone(),
          array::from_fn(|i| letter.has_idx_unchecked(i).then(|| node.clone())),
        );
        node.parents.insert(dawg.clone());
        dawg
      },
    )
  }

  pub fn str(word: &str) -> Self {
    Self::word(word.try_into().unwrap())
  }

  pub fn all(len: usize) -> Self {
    Self::str(".".repeat(len).as_str())
  }

  pub fn is_empty(&self) -> bool {
    self.children[26].is_none()
      && self.children[0..26]
        .iter()
        .all(|c| c.as_ref().map_or(false, |node| node.is_empty()))
  }

  pub fn is_leaf(&self) -> bool {
    self.children.iter().all(|c| c.is_none())
  }
}

impl From<()> for Dawg {
  fn from(_: ()) -> Self {
    Self::empty()
  }
}

impl From<Word> for Dawg {
  fn from(word: Word) -> Self {
    Self::word(word)
  }
}

impl From<&str> for Dawg {
  fn from(word: &str) -> Self {
    Self::str(word)
  }
}

/*
@note a trie may need unused branches pruned to reduce size or clear out
> prune, clear
*/
impl Node {
  pub fn prune(&mut self) {
    for child in self.children.iter_mut() {
      if let Some(node) = child.as_mut() {
        node.prune();
        if node.is_leaf() {
          child.take();
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
impl Node {}

/*
@note as tries could get quite large, they should support file operations:
> load_trie, save_trie, load_words, save_words, load, save
 > load and save try to decide the type of operation based on file extension

Trie file format:
a u32 number for each node, traversed in depth-first order
<5 unused bits><1 bit for whether the node is an end><26 bits for each letter>
*/
