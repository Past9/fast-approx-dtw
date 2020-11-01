use std::time::SystemTime;
mod fadtw;
use rand;
use rand::RngCore;
use std::thread;

use fadtw::*;

const STACK_SIZE: usize = 32 * 1024 * 1024;

/*
fn main() {
  let sig_y: [u8; SIG_SIZE] = [1, 3, 1, 5, 4, 3, 7, 2];
  let sig_x: [u8; SIG_SIZE] = [1, 1, 5, 1, 3, 8, 3, 2];

  let path = solve_dtw(sig_y, sig_x);

  for m in path.iter() {
    println!("{:?}", m);
  }
}
*/

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
    /*
    let err_map = gen_err_map(sig_y, sig_x, SIG_SIZE, &None);
    let path = map_paths(&err_map, SIG_SIZE, &None);
    */
    let path = solve_dtw(sig_y, sig_x);
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
