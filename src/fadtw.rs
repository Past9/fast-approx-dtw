use std::time::SystemTime;

pub const SIG_SIZE: usize = 4;
pub const NUM_DOWNSAMPLES: usize = 2; // 2nd logarithm of SIG_SIZE
pub const MAX_PATH_SIZE: usize = 9; // SIG_SIZE * 2 + 1

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
  Stop,
  Vertical,
  Horizontal,
  Diagonal,
}

pub struct MoveCosts {
  pub down: u32,
  pub left: u32,
  pub down_left: u32,
}

#[derive(Debug)]
pub struct Downsample {
  pub signal: [u8; SIG_SIZE],
  pub len: usize,
}
impl Downsample {
  #[inline]
  pub fn create_one(signal: &[u8; SIG_SIZE], len: usize) -> Downsample {
    let mut downsample = Downsample {
      signal: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
      len: len / 2,
    };

    for t in 0..downsample.len {
      downsample.signal[t] = signal[t * 2] / 2 + signal[t * 2 + 1] / 2
    }

    downsample
  }

  #[inline]
  pub fn create_all(signal: [u8; SIG_SIZE]) -> [Downsample; NUM_DOWNSAMPLES] {
    let mut downsamples: [Downsample; NUM_DOWNSAMPLES] =
      unsafe { std::mem::MaybeUninit::uninit().assume_init() };

    let mut last_downsample = Downsample {
      signal,
      len: SIG_SIZE,
    };

    for i in 0..NUM_DOWNSAMPLES {
      let downsample = Downsample::create_one(&last_downsample.signal, last_downsample.len);
      downsamples[NUM_DOWNSAMPLES - i - 1] = last_downsample;
      last_downsample = downsample;
    }

    downsamples
  }
}

#[derive(Debug)]
pub struct RelativePath {
  pub path: [Move; MAX_PATH_SIZE],
  pub path_len: usize,
}

#[inline]
pub fn solve_dtw(sig_y: [u8; SIG_SIZE], sig_x: [u8; SIG_SIZE]) -> RelativePath {
  let downsamples_y = Downsample::create_all(sig_y);
  let downsamples_x = Downsample::create_all(sig_x);

  let mut last_downsample_path: Option<RelativePath> = None;

  for ds in 0..NUM_DOWNSAMPLES {
    let err_map = gen_err_map(
      downsamples_y[ds].signal,
      downsamples_x[ds].signal,
      downsamples_y[ds].len,
      &last_downsample_path,
    );
    last_downsample_path = Some(get_optimal_path(err_map, downsamples_y[ds].len));
  }

  last_downsample_path.unwrap()
}

#[inline]
pub fn get_optimal_path(err_map: [[u32; SIG_SIZE]; SIG_SIZE], sample_size: usize) -> RelativePath {
  RelativePath {
    path: [Move::Diagonal; MAX_PATH_SIZE],
    path_len: sample_size - 1,
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PathCell {
  error: u32,
  to_parent: Move,
}

pub fn find_path(
  err_map: &[[u32; SIG_SIZE]; SIG_SIZE],
  sample_size: usize,
  downsample_path: &Option<RelativePath>,
) -> Vec<PathCell> {
  let mut path_map: [[PathCell; SIG_SIZE]; SIG_SIZE] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };

  /*
  calc_path_cell(
    sample_size - 2,
    sample_size - 2,
    &mut path_map,
    &err_map,
    sample_size,
  );
  */

  match downsample_path {
    Some(dp) => {
      // If we have a downsample path, then we only calculate possible paths through
      // the cells adjacent to it.
    }
    None => {
      // If there's no downsample path, then we're calculating paths
      // for every cell in the grid. We go right-to-left, top-to-bottom.
      for my in 0..sample_size {
        let y = sample_size - my - 1;
        for mx in 0..sample_size {
          let x = sample_size - mx - 1;
          calc_path_cell(y, x, &mut path_map, &err_map, sample_size);
        }
      }
    }
  };

  let mut final_path = Vec::with_capacity(MAX_PATH_SIZE);

  let mut x = 0;
  let mut y = 0;
  loop {
    final_path.push(path_map[y][x]);
    match path_map[y][x].to_parent {
      Move::Vertical => {
        y += 1;
      }
      Move::Horizontal => {
        x += 1;
      }
      Move::Diagonal => {
        y += 1;
        x += 1;
      }
      Move::Stop => {
        break;
      }
    }
  }

  final_path.shrink_to_fit();
  final_path
}

pub fn calc_path_cell(
  x: usize,
  y: usize,
  path_map: &mut [[PathCell; SIG_SIZE]; SIG_SIZE],
  err_map: &[[u32; SIG_SIZE]; SIG_SIZE],
  sample_size: usize,
) {
  if y == sample_size - 1 && x == sample_size - 1 {
    path_map[y][x] = PathCell {
      error: err_map[y][x],
      to_parent: Move::Stop,
    };
    return;
  }

  if y == sample_size - 1 {
    path_map[y][x] = PathCell {
      error: err_map[y][x] + path_map[y][x + 1].error,
      to_parent: Move::Horizontal,
    };
    return;
  }

  if x == sample_size - 1 {
    path_map[y][x] = PathCell {
      error: err_map[y][x] + path_map[y + 1][x].error,
      to_parent: Move::Vertical,
    };
    return;
  }

  let top_err = path_map[y + 1][x].error;
  let right_err = path_map[y][x + 1].error;
  let diag_err = path_map[y + 1][x + 1].error;

  let min_err = std::cmp::min(top_err, std::cmp::min(right_err, diag_err));

  if diag_err == min_err {
    path_map[y][x] = PathCell {
      error: err_map[y][x] + diag_err,
      to_parent: Move::Diagonal,
    };
    return;
  }

  if top_err == min_err {
    path_map[y][x] = PathCell {
      error: err_map[y][x] + top_err,
      to_parent: Move::Vertical,
    };
    return;
  }

  if right_err == min_err {
    path_map[y][x] = PathCell {
      error: err_map[y][x] + right_err,
      to_parent: Move::Horizontal,
    };
    return;
  }
}

#[inline]
pub fn gen_err_map(
  sig_y: [u8; SIG_SIZE],
  sig_x: [u8; SIG_SIZE],
  sample_size: usize,
  downsample_path: &Option<RelativePath>,
) -> [[u32; SIG_SIZE]; SIG_SIZE] {
  // Allocate space for the error map
  let mut err_map: [[u32; SIG_SIZE]; SIG_SIZE] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };

  // If we're building a subsample map, then there are uninitialized
  // values to the top and right of the top-right cell. We need to set
  // them to infinity so they don't mess up our min calculations later.
  if sample_size < SIG_SIZE {
    err_map[sample_size][sample_size - 2] = std::u32::MAX;
    err_map[sample_size][sample_size - 1] = std::u32::MAX;
    err_map[sample_size][sample_size] = std::u32::MAX;
    err_map[sample_size - 1][sample_size] = std::u32::MAX;
    err_map[sample_size - 2][sample_size] = std::u32::MAX;
  }

  match downsample_path {
    Some(dp) => {
      // We always have to calculate errors for the 4 cells near the
      // origin
      calc_err_cell(0, 0, &sig_y, &sig_x, &mut err_map); // Corner
      calc_err_cell(0, 1, &sig_y, &sig_x, &mut err_map); // Top
      calc_err_cell(1, 0, &sig_y, &sig_x, &mut err_map); // Right
      calc_err_cell(1, 1, &sig_y, &sig_x, &mut err_map); // Top-right

      // Now follow the downsample path through the map in reverse
      // (from the beginning of the signals), only
      // calculating error values for adjacent cells.

      // Coordinates of the current path step on the upsample
      let mut x = 1;
      let mut y = 1;
      let mut last_move: Option<Move> = None;

      for p in 0..dp.path_len {
        match dp.path[dp.path_len - p - 1] {
          Move::Vertical => {
            // Going up
            y += 2;

            // Set 3 boundary cells to "infinity",
            // unless we're by the left edge
            if x > 1 {
              err_map[y][x - 2] = std::u32::MAX;
              err_map[y - 1][x - 2] = std::u32::MAX;

              // only set this one if we didn't move
              // right just before this, because if we did,
              // then we'll overwrite a previously calculated
              // cell.
              match last_move {
                Some(Move::Horizontal) => {
                  err_map[y - 2][x - 2] = std::u32::MAX;
                }
                _ => {}
              };
            }

            // Set 4 candidate blocks
            calc_err_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x, &sig_y, &sig_x, &mut err_map);
          }
          Move::Horizontal => {
            // Going right
            x += 2;

            // Set 3 boundary cells to "infinity",
            // unless we're by the bottom edge
            if y > 1 {
              err_map[y - 2][x] = std::u32::MAX;
              err_map[y - 2][x - 1] = std::u32::MAX;

              // only set this one if we didn't move
              // up just before this, because if we did,
              // then we'll overwrite a previously calculated
              // cell.
              match last_move {
                Some(Move::Vertical) => {
                  err_map[y - 2][x - 2] = std::u32::MAX;
                }
                _ => {}
              };

              // Set 4 candidate blocks
              calc_err_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
              calc_err_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
              calc_err_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
              calc_err_cell(y, x, &sig_y, &sig_x, &mut err_map);
            }
          }
          Move::Diagonal => {
            // Going up and right
            y += 2;
            x += 2;

            // Set 4 boundary cells to "infinity"
            err_map[y][x - 2] = std::u32::MAX;
            err_map[y - 1][x - 3] = std::u32::MAX;
            err_map[y - 2][x] = std::u32::MAX;
            err_map[y - 3][x - 1] = std::u32::MAX;

            // Set 6 candidate blocks
            calc_err_cell(y - 2, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y - 1, x - 2, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x, &sig_y, &sig_x, &mut err_map);
          }
          Move::Stop => panic!("Invalid move"), // This variant doesn't apply here
        };

        // Set 5 boundary cells to the top and right
        if y < SIG_SIZE - 1 {
          err_map[y + 1][x - 1] = std::u32::MAX;
          err_map[y + 1][x] = std::u32::MAX;
        }

        if y < SIG_SIZE - 1 && x < SIG_SIZE - 1 {
          err_map[y + 1][x + 1] = std::u32::MAX;
        }

        if x < SIG_SIZE - 1 {
          err_map[y][x + 1] = std::u32::MAX;
          err_map[y - 1][x + 1] = std::u32::MAX;
        }

        last_move = Some(dp.path[p]);
      }
    }
    None => {
      // If we weren't given a downsample path, we fill out the
      // error map completely.
      for y in 0..sample_size {
        for x in 0..sample_size {
          calc_err_cell(y, x, &sig_y, &sig_x, &mut err_map);
        }
      }
    }
  }

  err_map
}

#[inline]
pub fn calc_err_cell(
  y: usize,
  x: usize,
  sig_y: &[u8; SIG_SIZE],
  sig_x: &[u8; SIG_SIZE],
  err_map: &mut [[u32; SIG_SIZE]; SIG_SIZE],
) {
  let err: u32 = (sig_y[y] as i16 - sig_x[x] as i16).abs() as u32;
  let left: u32 = match x == 0 {
    true => std::u32::MAX,
    false => err_map[y][x - 1],
  };
  let down: u32 = match y == 0 {
    true => std::u32::MAX,
    false => err_map[y - 1][x],
  };
  let down_left: u32 = match y == 0 || x == 0 {
    true => std::u32::MAX,
    false => err_map[y - 1][x - 1],
  };

  let mut min = std::cmp::min(left, std::cmp::min(down, down_left));
  if min == std::u32::MAX {
    min = 0;
  }
  err_map[y][x] = err + min;
}
