#![feature(min_const_generics)]
#![no_std]

mod alloc;
mod downsample;
mod dtw_solver;

pub mod downsample_fns;
pub mod loss_fns;
pub mod path;
pub mod stack_vec;

pub use dtw_solver::DtwSolver;
pub use path::{Move, Path, PathPoint};
pub use stack_vec::{StackVec, StackVecIterator};
