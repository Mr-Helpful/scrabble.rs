extern crate scrabble;
use scrabble::trie_ptr::Trie;

fn main() {
  println!("Hello, world!");

  // this is too slow.
  let t1 = Trie::str(".......");
  println!("{}", t1.is_empty());
}
