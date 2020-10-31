#![feature(min_const_generics)]

#[macro_use]
mod fadtw;
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
  let sig_y = rand_signal::<SIG_SIZE>();
  let sig_x = rand_signal::<SIG_SIZE>();
  let start = SystemTime::now();
  for _ in 0..512 {
    let path = solve_dtw(sig_y, sig_x);
  }
  let elapsed = SystemTime::now().duration_since(start);

  println!("total {:?} μs", elapsed.unwrap().as_nanos() as f32 / 1000.0);
}

fn rand_signal<const N: usize>() -> [u8; N] {
  let mut sig = [0u8; N];
  for i in 0..N {
    rand::thread_rng().fill_bytes(&mut sig);
  }
  sig
}
