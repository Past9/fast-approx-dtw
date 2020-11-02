#![feature(min_const_generics)]
#![no_std]

mod alloc;
mod downsample;
mod dtw_solver;
mod path;
mod stack_vec;

pub mod downsample_fns;
pub mod loss_fns;

pub use dtw_solver::DtwSolver;
