use std::time::SystemTime;

pub const SIG_SIZE: usize = 512;
pub const NUM_DOWNSAMPLES: usize = 9; // 2nd logarithm of SIG_SIZE
pub const MAX_PATH_SIZE: usize = 1025; // SIG_SIZE * 2 + 1

#[derive(Clone, Copy, Debug)]
pub enum Move {
  Down,
  Left,
  DownLeft,
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
  let start = SystemTime::now();
  let downsamples_y = Downsample::create_all(sig_y);
  let downsamples_x = Downsample::create_all(sig_x);

  let mut last_downsample_path: Option<RelativePath> = None;

  for ds in 0..NUM_DOWNSAMPLES {
    //println!("{:?}", downsamples_y[ds].len);
    let err_map = gen_err_map(
      downsamples_y[ds].signal,
      downsamples_x[ds].signal,
      downsamples_y[ds].len,
      &last_downsample_path,
    );
    last_downsample_path = Some(get_optimal_path(err_map, downsamples_y[ds].len));
  }

  let retval = last_downsample_path.unwrap();

  let elapsed = SystemTime::now().duration_since(start);
  //println!("{:?} μs", elapsed.unwrap().as_nanos() as f32 / 1000.0);

  //last_downsample_path.unwrap()
  retval
}

#[inline]
pub fn get_optimal_path(err_map: [[u32; SIG_SIZE]; SIG_SIZE], sample_size: usize) -> RelativePath {
  RelativePath {
    path: [Move::DownLeft; MAX_PATH_SIZE],
    path_len: sample_size - 1,
  }
}

#[inline]
pub fn gen_err_map(
  sig_y: [u8; SIG_SIZE],
  sig_x: [u8; SIG_SIZE],
  sample_size: usize,
  downsample_path: &Option<RelativePath>,
) -> [[u32; SIG_SIZE]; SIG_SIZE] {
  let mut ct_inf: usize = 0;
  let mut ct_err: usize = 0;

  // Allocate space for the error map
  let mut err_map: [[u32; SIG_SIZE]; SIG_SIZE] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };

  match downsample_path {
    Some(dp) => {
      // We always have to calculate errors for the 4 cells near the
      // origin
      calc_cell(0, 0, &sig_y, &sig_x, &mut err_map); // Corner
      calc_cell(0, 1, &sig_y, &sig_x, &mut err_map); // Top
      calc_cell(1, 0, &sig_y, &sig_x, &mut err_map); // Right
      calc_cell(1, 1, &sig_y, &sig_x, &mut err_map); // Top-right

      // Now follow the downsample path through the map in reverse
      // (from the beginning of the signals), only
      // calculating error values for adjacent cells.

      // Coordinates of the current path step on the upsample
      let mut x = 1;
      let mut y = 1;
      let mut last_move: Option<Move> = None;

      for p in 0..dp.path_len {
        match dp.path[dp.path_len - p - 1] {
          Move::Down => {
            // Going up
            y += 2;

            // Set 3 boundary cells to "infinity",
            // unless we're by the left edge
            if x > 1 {
              err_map[y][x - 2] = std::u32::MAX;
              err_map[y - 1][x - 2] = std::u32::MAX;

              ct_inf += 2;

              // only set this one if we didn't move
              // right just before this, because if we did,
              // then we'll overwrite a previously calculated
              // cell.
              match last_move {
                Some(Move::Left) => {
                  err_map[y - 2][x - 2] = std::u32::MAX;
                  ct_inf += 1;
                }
                _ => {}
              };
            }

            // Set 4 candidate blocks
            calc_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
            calc_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_cell(y, x, &sig_y, &sig_x, &mut err_map);
            ct_err += 4;
          }
          Move::Left => {
            // Going right
            x += 2;

            // Set 3 boundary cells to "infinity",
            // unless we're by the bottom edge
            if y > 1 {
              err_map[y - 2][x] = std::u32::MAX;
              err_map[y - 2][x - 1] = std::u32::MAX;
              ct_inf += 2;

              // only set this one if we didn't move
              // up just before this, because if we did,
              // then we'll overwrite a previously calculated
              // cell.
              match last_move {
                Some(Move::Down) => {
                  err_map[y - 2][x - 2] = std::u32::MAX;
                  ct_inf += 1;
                }
                _ => {}
              };

              // Set 4 candidate blocks
              calc_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
              calc_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
              calc_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
              calc_cell(y, x, &sig_y, &sig_x, &mut err_map);
              ct_err += 4;
            }
          }
          Move::DownLeft => {
            // Going up and right
            y += 2;
            x += 2;

            // Set 4 boundary cells to "infinity"
            err_map[y][x - 2] = std::u32::MAX;
            err_map[y - 1][x - 3] = std::u32::MAX;
            err_map[y - 2][x] = std::u32::MAX;
            err_map[y - 3][x - 1] = std::u32::MAX;
            ct_inf += 4;

            // Set 6 candidate blocks
            calc_cell(y - 2, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_cell(y - 1, x - 2, &sig_y, &sig_x, &mut err_map);
            calc_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
            calc_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_cell(y, x, &sig_y, &sig_x, &mut err_map);
            ct_err += 6;
          }
        };
        last_move = Some(dp.path[p]);
      }
    }
    None => {
      // If we weren't given a downsample path, we fill out the
      // error map completely.
      for y in 0..sample_size {
        for x in 0..sample_size {
          calc_cell(y, x, &sig_y, &sig_x, &mut err_map);
          ct_err += 1;
        }
      }
    }
  }

  err_map
}

#[inline]
pub fn calc_cell(
  y: usize,
  x: usize,
  sig_y: &[u8; SIG_SIZE],
  sig_x: &[u8; SIG_SIZE],
  err_map: &mut [[u32; SIG_SIZE]; SIG_SIZE],
) {
  let err: u32 = (sig_y[y] as i16 - sig_x[x] as i16).abs() as u32;
  let left: u32 = match x == 0 {
    true => 0,
    false => err_map[y][x - 1],
  };
  let down: u32 = match y == 0 {
    true => 0,
    false => err_map[y - 1][x],
  };
  let diag: u32 = match y == 0 || x == 0 {
    true => 0,
    false => err_map[y - 1][x - 1],
  };

  let mut min = std::cmp::min(left, std::cmp::min(down, diag));
  if min == std::u32::MAX {
    min = 0;
  }
  err_map[y][x] = err + min;
}

/*
pub fn gen_downsamples(sig: [u8; SIG_SIZE]) -> [Downsample; NUM_DOWNSAMPLES] {
  let mut downsamples: [Downsample; NUM_DOWNSAMPLES] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };

  let mut last_downsample = sig;
  for i in 0..NUM_DOWNSAMPLES {
    let downsample = downsample(&last_downsample);
    downsamples[NUM_DOWNSAMPLES - i - 1] = last_downsample;
    last_downsample = downsample;
  }

  downsamples
}

fn downsample<const S: usize>(signal: &[u8; S]) -> [u8; S] {
  let mut half = [0u8; S];

  for t in 0..(S / 2) {
    half[t] = signal[t * 2] / 2 + signal[t * 2 + 1] / 2
  }

  half
}
*/
