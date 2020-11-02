#![feature(min_const_generics)]

mod alloc;
mod fadtw;
mod stack_vec;

use rand;
use rand::RngCore;
use std::thread;
use std::time::SystemTime;

use fadtw::*;

const STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() {
  let child = thread::Builder::new()
    .stack_size(STACK_SIZE)
    .spawn(run)
    .unwrap();

  child.join().unwrap();
}

fn run() {
  let start = SystemTime::now();
  let sig_y = rand_signal();
  let sig_x = rand_signal();
  for _ in 0..576 {
    let path = fast_approx_dtw(sig_y, sig_x);
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
