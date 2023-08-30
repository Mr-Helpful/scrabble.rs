use super::*;
use rand::distributions::{Distribution, Standard};

impl Trie {
  pub fn random<R: rand::Rng + ?Sized>(rng: &mut R, branch_p: f64, len: usize) -> Trie {
    // we don't want to generate infinite tries, so we limit the maximum
    // possible depth of the trie
    if len == 0 {
      return Trie {
        is_end: false,
        children: array::from_fn(|_| None),
      };
    }

    Trie {
      is_end: rng.gen(),
      children: array::from_fn(|_| {
        if rng.gen_bool(branch_p) {
          Some(Box::new(Self::random(rng, branch_p, len - 1)))
        } else {
          None
        }
      }),
    }
  }
}

impl Distribution<Trie> for Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Trie {
    Trie::random(rng, 0.25, 20)
  }
}
