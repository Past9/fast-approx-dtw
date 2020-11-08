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

  pub fn get_disparity<const SIGNAL_SIZE: usize>(&self) -> [f32; SIGNAL_SIZE] {
    let mut deviation: [f32; SIGNAL_SIZE] = alloc(false);

    let mut d = 0f32;
    let mut t = 0;
    for point in self.iter() {
      match point.to_parent {
        Move::Diagonal => {
          t += 1;
        }
        Move::Horizontal => {
          t += 1;
          d -= 1f32;
        }
        Move::Vertical => {
          d += 1f32;
        }
        _ => {
          break;
        }
      }
      deviation[t] = d;
    }

    deviation
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
  pub loss: f32,
  pub to_parent: Move,
}
