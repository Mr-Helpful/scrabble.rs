use std::array;

use super::{Letter, Word};
use rand::distributions::{Distribution, Standard};

impl Letter {
  fn random<R: rand::Rng + ?Sized>(rng: &mut R, char_p: f64) -> Letter {
    Letter(array::from_fn(|_| rng.gen_bool(char_p)))
  }
}

impl Distribution<Letter> for Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Letter {
    Letter::random(rng, 0.25)
  }
}

impl Word {
  pub fn random<R: rand::Rng + ?Sized>(
    rng: &mut R,
    char_p: f64,
    group_p: f64,
    max_len: usize,
  ) -> Word {
    let geo_len = rng.gen::<f64>().log(group_p).floor();
    let len = (geo_len as usize).min(max_len);
    Word((0..len).map(|_| Letter::random(rng, char_p)).collect())
  }
}

impl Distribution<Word> for Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Word {
    Word::random(rng, 0.25, 0.9, 20)
  }
}
