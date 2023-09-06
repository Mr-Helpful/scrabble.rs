#[cfg(test)]
use super::{Trie, Word};

#[cfg(test)]
mod collection_tests {
  use super::*;

  mod empty {
    use super::*;

    #[test]
    fn is_leaf() {
      // an empty trie should be a leaf
      let trie = Trie::empty();
      assert!(trie.is_leaf());
    }

    #[test]
    fn is_empty() {
      // an empty trie should have no words
      let trie = Trie::empty();
      assert!(trie.is_empty());
    }

    #[test]
    fn len_0() {
      // an empty trie shold be of length 0
      let trie = Trie::empty();
      assert_eq!(trie.len(), 0);
    }
  }

  /* @note Trie::word testing */
  mod word {
    use super::*;

    #[test]
    fn empty() {
      // an empty word should produce a trie that has an end but is a leaf
      let trie = Trie::str("");
      assert!(trie.is_end);
      assert!(trie.is_leaf());
    }

    #[test]
    fn single_char() {
      // a single character will produce an iterator with only that character
      let mut trie = Trie::str("a");
      assert_eq!(trie.next(), Some(String::from("a")));
      assert_eq!(trie.next(), None);
    }

    #[test]
    fn single_word() {
      // a single word will produce an iterator with only that word
      let mut trie = Trie::str("hello");
      assert_eq!(trie.next(), Some(String::from("hello")));
      assert_eq!(trie.next(), None);
    }

    #[test]
    fn wildcard() {
      // the '.' wildcard should match every letter
      let trie = Trie::str(".");
      assert!(trie.eq(('a'..='z').map(String::from)));
    }

    #[test]
    fn ranges() {
      // we should be able to represent character ranges
      let trie = Trie::str("[f-m]");
      assert!(trie.eq(('f'..='m').map(String::from)));
    }

    #[test]
    fn char_group() {
      // we should be able to represent character groups
      let mut trie = Trie::str("[fiep]");

      assert_eq!(trie.next(), Some(String::from("e")));
      assert_eq!(trie.next(), Some(String::from("f")));
      assert_eq!(trie.next(), Some(String::from("i")));
      assert_eq!(trie.next(), Some(String::from("p")));
      assert_eq!(trie.next(), None);
    }

    #[test]
    fn char_range_group() {
      // we should be able to use multiple ranges in a group
      let mut trie = Trie::str("[f-hp-t]");

      assert_eq!(trie.next(), Some(String::from("f")));
      assert_eq!(trie.next(), Some(String::from("g")));
      assert_eq!(trie.next(), Some(String::from("h")));
      assert_eq!(trie.next(), Some(String::from("p")));

      assert_eq!(trie.next(), Some(String::from("q")));
      assert_eq!(trie.next(), Some(String::from("r")));
      assert_eq!(trie.next(), Some(String::from("s")));
      assert_eq!(trie.next(), Some(String::from("t")));

      assert_eq!(trie.next(), None);
    }

    #[test]
    fn mixed_group() {
      // we should be able to use a mix of ranges and characters in a group
      let mut trie = Trie::str("[f-hmop-t]");

      assert_eq!(trie.next(), Some(String::from("f")));
      assert_eq!(trie.next(), Some(String::from("g")));
      assert_eq!(trie.next(), Some(String::from("h")));

      assert_eq!(trie.next(), Some(String::from("m")));
      assert_eq!(trie.next(), Some(String::from("o")));

      assert_eq!(trie.next(), Some(String::from("p")));
      assert_eq!(trie.next(), Some(String::from("q")));
      assert_eq!(trie.next(), Some(String::from("r")));
      assert_eq!(trie.next(), Some(String::from("s")));
      assert_eq!(trie.next(), Some(String::from("t")));

      assert_eq!(trie.next(), None);
    }

    #[test]
    fn two_group() {
      // using two groups should give the cartesian product of both
      let mut trie = Trie::str("[ab][cd]");
      assert_eq!(trie.next(), Some(String::from("ac")));
      assert_eq!(trie.next(), Some(String::from("ad")));
      assert_eq!(trie.next(), Some(String::from("bc")));
      assert_eq!(trie.next(), Some(String::from("bd")));
      assert_eq!(trie.next(), None);
    }

    #[test]
    fn mixed_all() {
      // using mixtures of groups, ranges and character should work
      let mut trie = Trie::str("[ab]c[d-f]");
      assert_eq!(trie.next(), Some(String::from("acd")));
      assert_eq!(trie.next(), Some(String::from("ace")));
      assert_eq!(trie.next(), Some(String::from("acf")));

      assert_eq!(trie.next(), Some(String::from("bcd")));
      assert_eq!(trie.next(), Some(String::from("bce")));
      assert_eq!(trie.next(), Some(String::from("bcf")));
      assert_eq!(trie.next(), None);
    }
  }
}

#[cfg(test)]
mod set_tests {
  use rand::{distributions::Standard, prelude::Distribution, thread_rng, Rng};

  use super::*;

  fn assert_all<T, F>(test: F, no_samples: usize)
  where
    F: Fn(T) -> bool,
    Standard: Distribution<T>,
  {
    // asserts a test is true on multiple samples
    let mut rng = thread_rng();
    assert!((&mut rng).sample_iter(Standard).take(no_samples).all(test))
  }

  fn assert_strs<F: Fn(&str) -> bool>(test: F, no_samples: usize, max_len: usize) {
    assert_all(
      |word: Word| {
        let mut s = word.to_string();
        s.truncate(max_len);
        test(s.as_str())
      },
      no_samples,
    )
  }

  mod has {
    use super::*;

    #[test]
    fn nothing() {
      // an empty trie should match no words
      // we can't test all words possible to generate,
      // so we test 10,000 sampled words up to 20 letters long.
      let trie = Trie::empty();
      assert_strs(|word| !trie.has(word), 1, 15)
    }

    #[test]
    fn empty() {
      // a trie generated from a empty word should contain the empty word
      let trie = Trie::str("");
      assert!(trie.has(""))
    }

    #[test]
    fn char() {
      // a trie generated from a single character should contain that character
      for c in 'a'..='z' {
        let s = String::from(c);
        let word = s.as_str();
        let trie = Trie::str(word);
        assert!(trie.has(word))
      }
    }

    #[test]
    fn word() {
      // a trie generated from a word should contain that word
      // we obviously can't test all possible words,
      // so we sample 10,000 random words up to 20 letters long
      assert_strs(|word| Trie::str(word).has_all(word), 1, 15)
    }
  }

  mod add {
    use super::*;

    #[test]
    fn has() {
      // adding a word to a trie should result in a trie containing that word
      // again we sample 10,000 words of up to 20 letters long
      let trie = Trie::empty();
      assert_strs(|word| (&trie + word).has_all(word), 1, 15)
    }
  }

  mod sub {
    use super::*;

    #[test]
    fn empty_sub() {
      // subtracting anything from the empty trie should have no change
      let trie = Trie::empty();
      assert_strs(|word| (&trie - word) == Trie::empty(), 1, 15)
    }

    #[test]
    fn add_sub() {
      // adding and then subtracting a word should have no change
      let trie = Trie::empty();
      assert_strs(|word| &(&trie + word) - word == Trie::empty(), 1, 15)
    }

    #[test]
    fn word_sub() {
      // subtracting a word from a word trie should give the empty trie
      assert_strs(|word| &Trie::str(word) - word == Trie::empty(), 1, 15)
    }
  }

  mod or {
    use super::*;

    #[test]
    fn or_unit() {
      // the empty trie should be the unit of or
      assert_all(|trie| &trie | Trie::empty() == trie, 1)
    }
  }
}
