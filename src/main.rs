#![feature(min_const_generics)]

#[macro_use]
mod alloc;
mod downsample;
mod fadtw;
mod path;
mod stack_vec;

use rand;
use rand::RngCore;
use std::thread;
use std::time::SystemTime;

use fadtw::*;

const STACK_SIZE: usize = 128 * 1024 * 1024;

pub const SIG_SIZE: usize = 1024;

pub fn downsample_fn(sig_y: &u8, sig_x: &u8) -> u8 {
  sig_y / 2 + sig_x / 2
}

pub fn err_fn(sig_y: &u8, sig_x: &u8) -> u32 {
  (*sig_y as i16 - *sig_x as i16).abs() as u32
}

fn main() {
  let child = thread::Builder::new()
    .stack_size(STACK_SIZE)
    .spawn(run)
    .unwrap();

  child.join().unwrap();
}

fn run() {
  let sig_y = rand_signal();
  let sig_x = rand_signal();

  let start = SystemTime::now();
  for _ in 0..57600 {
    let mut dtw_solver = DtwSolver::<u8, 2049>::new(&sig_y, &sig_x, downsample_fn, err_fn);
    dtw_solver.solve();
    //fast_approx_dtw::<u8, 2049>(&sig_y, &sig_x, downsample_fn, err_fn);
  }
  let elapsed = SystemTime::now().duration_since(start);

  println!("total {:?} μs", elapsed.unwrap().as_nanos() as f32 / 1000.0);
}

fn rand_signal() -> [u8; SIG_SIZE] {
  let mut sig = [0u8; SIG_SIZE];
  for i in 0..SIG_SIZE {
    rand::thread_rng().fill_bytes(&mut sig);
  }
  sig
}
