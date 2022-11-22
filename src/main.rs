mod trie;
mod trie_paths;
use crate::trie::Trie;

fn bit_range(s: u8, e: u8) -> u32 {
  (1 << e + 1) - (1 << s)
}

fn main() {
  println!("Hello, world!");

  let t1 = Trie::empty();
  println!("{}", t1.is_empty());
}
