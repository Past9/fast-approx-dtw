use crate::stack_vec::StackVec;

pub type Path<const N: usize> = StackVec<PathPoint, N>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
  Stop,
  Vertical,
  Horizontal,
  Diagonal,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PathPoint {
  pub error: u32,
  pub to_parent: Move,
}
impl PathPoint {
  pub fn empty() -> PathPoint {
    PathPoint {
      error: 0,
      to_parent: Move::Stop,
    }
  }
}
