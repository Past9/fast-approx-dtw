#[macro_use]
mod fadtw;
use rand;
use rand::RngCore;
use std::thread;
use std::time::SystemTime;

use fadtw::*;

const STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() {
  let sig_y: [u8; SIG_SIZE] = [1, 3, 1, 5];
  let sig_x: [u8; SIG_SIZE] = [1, 1, 5, 1];

  let err_map = gen_err_map(sig_y, sig_x, SIG_SIZE, &None);
  let path_map = find_path(&err_map, SIG_SIZE, &None);

  println!("{:?}", path_map);

  //let tc = get_paths(1, 1, &err_map, 0);

  //println!("{:?}", tc);
}

/*
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
*/
