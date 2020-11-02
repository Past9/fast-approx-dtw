/*
    Time how long it would take to depth-map two stereo 640 x 480 images.
*/
#![feature(min_const_generics)]
use rand::Rng;
use std::thread;
use std::time::SystemTime;

use fast_approx_dtw::{downsample_fns, loss_fns, DtwSolver};

const STACK_SIZE: usize = 8 * 1024 * 1024;
const IMG_HEIGHT: usize = 640;
const IMG_WIDTH: usize = 480;

fn main() {
  // Run in a separate thread so we can control the stack size.
  thread::Builder::new()
    .stack_size(STACK_SIZE)
    .spawn(run)
    .unwrap()
    .join()
    .unwrap();
}

fn run() {
  let left_line = rand_line::<IMG_WIDTH>();
  let right_line = rand_line::<IMG_WIDTH>();

  let start = SystemTime::now();
  for _ in 0..IMG_HEIGHT {
    let path = DtwSolver::<[u8; 3], IMG_WIDTH, 1281>::new(
      &left_line,
      &right_line,
      downsample_fns::mean,
      loss_fns::euclidean_u8::<3>,
    )
    .solve();
  }
  let elapsed = SystemTime::now().duration_since(start);

  println!(
    "total {:?} μs",
    SystemTime::now().duration_since(start).unwrap().as_nanos() as f32 / 1000.0
  );
}

fn rand_line<const N: usize>() -> [[u8; 3]; N] {
  let mut rng = rand::thread_rng();
  let mut sig = [[0u8; 3]; N];
  for i in 0..N {
    sig[i] = [rng.gen(), rng.gen(), rng.gen()];
  }
  sig
}
