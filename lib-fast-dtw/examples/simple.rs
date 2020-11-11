/*
    Calculate the optimal transform between two 4-sample signals.
*/
use fast_dtw::{downsample_fns, loss_fns, DtwSolver};

fn main() {
    let sig_y = [1f32, 3f32, 1f32, 5f32];
    let sig_x = [1f32, 1f32, 5f32, 1f32];

    let path = DtwSolver::<f32, 4, 9>::new(&sig_y, &sig_x, downsample_fns::mean_u8, loss_fns::dist)
        .solve();

    for path_move in path.iter() {
        println!("{:?}", path_move);
    }
}
