use crate::alloc::alloc;
use crate::{stack_vec::StackVec, StackVecIterator};

//pub type Path<const N: usize> = StackVec<PathPoint, N>;

pub struct Path<const N: usize>(StackVec<PathPoint, N>);
impl<const N: usize> Path<N> {
  pub fn empty(zero_mem: bool) -> Path<N> {
    Path(StackVec::empty(zero_mem))
  }

  pub fn iter(&self) -> StackVecIterator<PathPoint, N> {
    self.0.iter()
  }

  pub fn push(&mut self, item: PathPoint) {
    self.0.push(item);
  }

  pub fn warp<SampleType: Copy, const SIGNAL_SIZE: usize>(
    &self,
    signal: [SampleType; SIGNAL_SIZE],
  ) -> [SampleType; SIGNAL_SIZE] {
    let mut warped: [SampleType; SIGNAL_SIZE] = alloc(false);

    let mut t_signal = 0;
    let mut t_warped = 0;
    for point in self.iter() {
      match point.to_parent {
        Move::Diagonal => {
          t_signal += 1;
          t_warped += 1;
        }
        Move::Horizontal => {
          t_warped += 1;
        }
        Move::Vertical => {
          t_signal += 1;
        }
        _ => {
          break;
        }
      }

      warped[t_warped] = signal[t_signal];
    }

    warped
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
  Stop,
  Vertical,
  Horizontal,
  Diagonal,
}

#[derive(Debug, Copy, Clone)]
pub struct PathPoint {
  pub loss: u32,
  pub to_parent: Move,
}
