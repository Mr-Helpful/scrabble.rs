// mod dawg;
mod trie_ptr;
mod word;
use crate::trie_ptr::Trie;

fn main() {
  println!("Hello, world!");

  // this is too slow.
  let t1 = Trie::str(".......");
  println!("{}", t1.is_empty());
}
