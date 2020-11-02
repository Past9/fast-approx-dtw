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
  pub loss: u32,
  pub to_parent: Move,
}
