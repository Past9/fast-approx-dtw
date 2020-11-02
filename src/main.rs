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

//pub const SIG_SIZE: usize = 1024;

pub fn downsample_fn(sig_y: &u8, sig_x: &u8) -> u8 {
  sig_y / 2 + sig_x / 2
}

pub fn loss_fn(sig_y: &u8, sig_x: &u8) -> u32 {
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
  for _ in 0..576 {
    let path = DtwSolver::<u8, 1024, 2049>::new(&sig_y, &sig_x, downsample_fn, loss_fn).solve();
  }
  let elapsed = SystemTime::now().duration_since(start);

  println!("total {:?} μs", elapsed.unwrap().as_nanos() as f32 / 1000.0);
}

fn rand_signal() -> [u8; 1024] {
  let mut sig = [0u8; 1024];
  for i in 0..1024 {
    rand::thread_rng().fill_bytes(&mut sig);
  }
  sig
}
