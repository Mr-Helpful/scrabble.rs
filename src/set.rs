/// The mathematical notion of sets.
///
/// The default implementations are **not** efficient and should be overridden.
pub trait Set:
  Sized + Default + Clone + IntoIterator + FromIterator<Self::Item> + Extend<Self::Item>
where
  Self::Item: Eq,
{
  /*------------------------------*/
  /*-        Constructors        -*/
  /*------------------------------*/

  /// A Set that only contains one item.
  fn singleton(item: Self::Item) -> Self {
    std::iter::once(item).collect()
  }

  /*------------------------------*/
  /*-         Statistics         -*/
  /*------------------------------*/

  /// The number of items in the set.
  fn len(&self) -> usize {
    self.clone().into_iter().count()
  }
  /// Returns true if the set contains no items.
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
  /// Whether the set contains the item.
  fn contains(&self, item: &Self::Item) -> bool {
    self.clone().into_iter().any(|i| &i == item)
  }
  /// Whether this set is a subset of another set.
  fn subset(&self, other: &Self) -> bool {
    self.clone().into_iter().all(|item| other.contains(&item))
  }
  /// Whether this set is a superset of another set.
  fn superset(&self, other: &Self) -> bool {
    other.subset(self)
  }

  /*------------------------------*/
  /*-   Item Based Operations    -*/
  /*------------------------------*/

  /// Adds an item and returns whether the item previously existed.
  fn insert(&mut self, item: Self::Item) -> bool {
    if self.contains(&item) {
      return true;
    }
    self.extend(std::iter::once(item));
    false
  }
  /// Only keeps items in the set that satisfy the predicate.
  fn retain<F>(&mut self, f: F)
  where
    F: FnMut(&Self::Item) -> bool,
  {
    *self = self.clone().into_iter().filter(f).collect();
  }
  /// Removes an item and returns whether the item previously existed.
  fn delete(&mut self, item: &Self::Item) -> bool {
    if !self.contains(item) {
      return false;
    }
    self.retain(|i| i != item);
    true
  }
  /// Removes all items from the set.
  fn clear(&mut self) {
    *self = Self::default();
  }

  /*------------------------------*/
  /*-    Set Based Operations    -*/
  /*------------------------------*/

  /// Reduces this set to the items in common with another set.
  fn intersect(&mut self, other: &Self) {
    self.retain(|item| other.contains(item));
  }
  /// Removes all the items in another set
  fn remove(&mut self, other: &Self) {
    self.retain(|item| !other.contains(item));
  }
}
