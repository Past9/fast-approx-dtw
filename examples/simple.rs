/*
    Calculate the optimal transform between two 4-sample signals.
*/
use fast_approx_dtw::{downsample_fns, loss_fns, DtwSolver};

fn main() {
  let sig_y = [1, 3, 1, 5];
  let sig_x = [1, 1, 5, 1];

  let path =
    DtwSolver::<u8, 4, 9>::new(&sig_y, &sig_x, downsample_fns::mean_u8, loss_fns::dist_u8).solve();

  for path_move in path.iter() {
    println!("{:?}", path_move);
  }
}
