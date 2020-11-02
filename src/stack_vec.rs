use crate::alloc; //::alloc;

pub struct StackVec<T, const N: usize> {
  items: [T; N],
  len: usize,
}
impl<T, const N: usize> StackVec<T, N> {
  #[inline]
  pub fn empty(zero_mem: bool) -> StackVec<T, N> {
    StackVec {
      items: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
      len: 0,
    }
  }

  #[inline]
  pub fn new(items: [T; N], len: usize) -> StackVec<T, N> {
    StackVec { items, len }
  }

  #[inline]
  pub fn push(&mut self, item: T) {
    if self.len >= N {
      panic!("failed to push: StackVec is full (capacity: {})", N);
    }

    self.items[self.len] = item;
    self.len += 1;
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.len
  }

  #[inline]
  pub fn iter(&self) -> StackVecIterator<T, N> {
    StackVecIterator::new(&self)
  }
}
impl<T, const N: usize> core::ops::Index<usize> for StackVec<T, N> {
  type Output = T;

  #[inline]
  fn index(&self, index: usize) -> &Self::Output {
    if self.len == 0 || index > self.len - 1 {
      panic!(
        "index out of bounds: the len is {} but the index is {}",
        self.len, index
      );
    }

    &self.items[index]
  }
}
impl<T, const N: usize> core::ops::IndexMut<usize> for StackVec<T, N> {
  #[inline]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    if self.len == 0 || index > self.len - 1 {
      panic!(
        "index out of bounds: the len is {} but the index is {}",
        self.len, index
      );
    }

    &mut self.items[index]
  }
}

pub struct StackVecIterator<'a, T, const N: usize> {
  stack_vec: &'a StackVec<T, N>,
  pos: usize,
}
impl<'a, T, const N: usize> StackVecIterator<'a, T, N> {
  #[inline]
  pub fn new(stack_vec: &'a StackVec<T, N>) -> StackVecIterator<T, N> {
    StackVecIterator { stack_vec, pos: 0 }
  }
}
impl<'a, T, const N: usize> Iterator for StackVecIterator<'a, T, N> {
  type Item = &'a T;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match self.pos >= self.stack_vec.len {
      true => None,
      false => {
        let item = Some(&self.stack_vec[self.pos]);
        self.pos += 1;
        item
      }
    }
  }
}
impl<'a, T, const N: usize> DoubleEndedIterator for StackVecIterator<'a, T, N> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    match self.pos >= self.stack_vec.len {
      true => None,
      false => {
        let item = Some(&self.stack_vec[self.stack_vec.len - self.pos - 1]);
        self.pos += 1;
        item
      }
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn keeps_item_count() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    assert_eq!(0, vec.len);
    vec.push(2);
    assert_eq!(1, vec.len);
    vec.push(3);
    assert_eq!(2, vec.len);
    vec.push(4);
    assert_eq!(3, vec.len);
  }

  #[test]
  fn can_fill_completely() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    vec.push(6);
  }

  #[test]
  #[should_panic(expected = "failed to push: StackVec is full (capacity: 5)")]
  fn panics_on_overfill() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    vec.push(6);
    vec.push(7);
  }

  #[test]
  fn is_indexable() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    assert_eq!(4, vec[2]);
    assert_eq!(3, vec[1]);
    assert_eq!(2, vec[0]);
  }

  #[test]
  #[should_panic(expected = "index out of bounds: the len is 0 but the index is 0")]
  fn panics_on_zero_index_when_empty() {
    StackVec::<u8, 5>::empty(false)[0];
  }

  #[test]
  #[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
  fn panics_on_index_above_len() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec[3];
  }

  #[test]
  fn iterates_items() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    let values = [1, 2, 3];

    for item in values.iter() {
      vec.push(*item);
    }

    for (i, item) in vec.iter().enumerate() {
      assert_eq!(values[i], *item);
    }
  }

  #[test]
  fn reverse_iterates_items() {
    let mut vec = StackVec::<u8, 5>::empty(false);
    let values = [1, 2, 3];

    for item in values.iter() {
      vec.push(*item);
    }

    for (i, item) in vec.iter().rev().enumerate() {
      assert_eq!(values[values.len() - i - 1], *item);
    }
  }
}
