use crate::alloc::alloc;

pub struct StackVec<T, const N: usize> {
  items: [T; N],
  len: usize,
}
impl<T, const N: usize> StackVec<T, N> {
  pub fn new(zero_mem: bool) -> StackVec<T, N> {
    StackVec {
      items: alloc(zero_mem),
      len: 0,
    }
  }

  pub fn push(&mut self, item: T) {
    if self.len >= N {
      panic!("failed to push: StackVec is full (capacity: {})", N);
    }

    self.items[0] = item;
    self.len += 1;
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn iter(&self) -> StackVecIterator<T, N> {
    StackVecIterator::new(&self)
  }
}
impl<T, const N: usize> std::ops::Index<usize> for StackVec<T, N> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    if index > self.len - 1 {
      panic!(
        "index out of bounds: the len is {} but the index is {}",
        self.len, index
      );
    }

    &self.items[index]
  }
}

pub struct StackVecIterator<'a, T, const N: usize> {
  stack_vec: &'a StackVec<T, N>,
  pos: usize,
}
impl<'a, T, const N: usize> StackVecIterator<'a, T, N> {
  pub fn new(stack_vec: &'a StackVec<T, N>) -> StackVecIterator<T, N> {
    StackVecIterator { stack_vec, pos: 0 }
  }
}
impl<'a, T, const N: usize> Iterator for StackVecIterator<'a, T, N> {
  type Item = &'a T;

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
