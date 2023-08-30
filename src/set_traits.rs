use std::{fmt::Debug, str::FromStr};

pub trait StringSet: FromStr + Clone
where
  <Self as FromStr>::Err: Debug,
{
  fn diff(&self, other: &Self) -> Self;
  fn and(&self, other: &Self) -> Self;
  fn or(&self, other: &Self) -> Self;
}
