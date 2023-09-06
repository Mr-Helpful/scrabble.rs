use std::array;

use super::Letter;
use rand::distributions::{Distribution, Standard};

impl Letter {
  pub(crate) fn random<R: rand::Rng + ?Sized>(rng: &mut R, char_p: f64) -> Letter {
    Letter(array::from_fn(|_| rng.gen_bool(char_p)))
  }
}

impl Distribution<Letter> for Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Letter {
    Letter::random(rng, 0.25)
  }
}
