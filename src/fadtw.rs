pub const SIG_SIZE: usize = 1024;
pub const NUM_DOWNSAMPLES: usize = 10; // 2nd logarithm of SIG_SIZE
pub const MAX_PATH_SIZE: usize = 2049; // SIG_SIZE * 2 + 1

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
  Stop,
  Vertical,
  Horizontal,
  Diagonal,
}

#[derive(Debug, Copy, Clone)]
pub struct Downsample {
  pub signal: [u8; SIG_SIZE],
  pub len: usize,
}
impl Downsample {
  #[inline]
  pub fn create_one(signal: &[u8; SIG_SIZE], len: usize) -> Downsample {
    let mut downsample = Downsample {
      signal: unsafe { std::mem::MaybeUninit::uninit().assume_init() }, // [0u8; SIG_SIZE],
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
    /*
    [Downsample {
      signal: [0u8; SIG_SIZE],
      len: 0,
    }; NUM_DOWNSAMPLES];
    */

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

#[inline]
pub fn solve_dtw(sig_y: [u8; SIG_SIZE], sig_x: [u8; SIG_SIZE]) -> Path {
  let downsamples_y = Downsample::create_all(sig_y);
  let downsamples_x = Downsample::create_all(sig_x);

  let mut last_downsample_path: Option<Path> = None;

  for ds in 0..NUM_DOWNSAMPLES {
    let err_map = gen_err_map(
      downsamples_y[ds].signal,
      downsamples_x[ds].signal,
      downsamples_y[ds].len,
      &last_downsample_path,
    );
    last_downsample_path = Some(map_paths(
      &err_map,
      downsamples_y[ds].len,
      &last_downsample_path,
    ));
    let ds_path = last_downsample_path.clone().unwrap();
    let x = 0;
  }

  last_downsample_path.unwrap()
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct PathPoint {
  error: u32,
  to_parent: Move,
}
impl PathPoint {
  pub fn empty() -> PathPoint {
    PathPoint {
      error: 0,
      to_parent: Move::Stop,
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Path {
  len: usize,
  moves: [PathPoint; MAX_PATH_SIZE],
}
impl Path {
  fn new(map: &[[PathPoint; SIG_SIZE]; SIG_SIZE]) -> Path {
    let mut y = 0;
    let mut x = 0;
    let mut len = 0;
    let mut moves: [PathPoint; MAX_PATH_SIZE] =
      unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    //[PathPoint::empty(); MAX_PATH_SIZE];
    let mut current_cell = map[y][x];

    loop {
      match current_cell.to_parent {
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
        Move::Stop => {}
      }

      moves[len] = current_cell;
      current_cell = map[y][x];
      len += 1;
      if current_cell.to_parent == Move::Stop {
        break;
      }
    }

    Path { len, moves }
  }

  pub fn iter(&self) -> PathMapIterator {
    PathMapIterator::new(self.len, &self.moves)
  }

  pub fn len(&self) -> usize {
    self.len
  }
}

pub struct PathMapIterator<'a> {
  len: usize,
  moves: &'a [PathPoint; MAX_PATH_SIZE],
  pos: usize,
}
impl<'a> PathMapIterator<'a> {
  fn new(len: usize, moves: &'a [PathPoint; MAX_PATH_SIZE]) -> PathMapIterator<'a> {
    PathMapIterator { len, moves, pos: 0 }
  }
}
impl<'a> Iterator for PathMapIterator<'a> {
  type Item = PathPoint;

  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.len {
      return None;
    } else {
      let item = Some(self.moves[self.pos]);
      self.pos += 1;
      return item;
    }
  }
}
impl<'a> DoubleEndedIterator for PathMapIterator<'a> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.pos >= self.len {
      return None;
    } else {
      let item = Some(self.moves[self.len - self.pos - 1]);
      self.pos += 1;
      return item;
    }
  }
}

pub fn map_paths(
  err_map: &[[u32; SIG_SIZE]; SIG_SIZE],
  sample_size: usize,
  downsample_path: &Option<Path>,
) -> Path {
  let mut path_map: [[PathPoint; SIG_SIZE]; SIG_SIZE] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };
  /*
  [[PathPoint {
    error: 0,
    to_parent: Move::Diagonal,
  }; SIG_SIZE]; SIG_SIZE];
  */

  match downsample_path {
    Some(dp) => {
      // If we have a downsample path, then we only calculate possible paths through
      // the cells adjacent to it.
      let mut y = sample_size - 1;
      let mut x = sample_size - 1;

      // Initialize the corner "Stop" cell and its 3 adjacent cells
      calc_path_cell(y, x, &mut path_map, &err_map, sample_size);
      calc_path_cell(y, x - 1, &mut path_map, &err_map, sample_size);
      calc_path_cell(y - 1, x, &mut path_map, &err_map, sample_size);
      calc_path_cell(y - 1, x - 1, &mut path_map, &err_map, sample_size);

      for path_move in dp.iter().rev() {
        //println!("YX {} {} {} {:?}", y, x, sample_size, path_move);
        match path_move.to_parent {
          Move::Vertical => {
            y -= 2;
            calc_path_cell(y, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y, x - 1, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x - 1, &mut path_map, &err_map, sample_size);
          }
          Move::Horizontal => {
            x -= 2;
            calc_path_cell(y, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y, x - 1, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x - 1, &mut path_map, &err_map, sample_size);
          }
          Move::Diagonal => {
            y -= 2;
            x -= 2;
            calc_path_cell(y + 1, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y, x + 1, &mut path_map, &err_map, sample_size);
            calc_path_cell(y, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y, x - 1, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x, &mut path_map, &err_map, sample_size);
            calc_path_cell(y - 1, x - 1, &mut path_map, &err_map, sample_size);
          }
          Move::Stop => {
            break;
          }
        }
      }
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

  Path::new(&path_map)
}

pub fn calc_path_cell(
  y: usize,
  x: usize,
  path_map: &mut [[PathPoint; SIG_SIZE]; SIG_SIZE],
  err_map: &[[u32; SIG_SIZE]; SIG_SIZE],
  sample_size: usize,
) {
  if y == sample_size - 1 && x == sample_size - 1 {
    path_map[y][x] = PathPoint {
      error: err_map[y][x],
      to_parent: Move::Stop,
    };
    return;
  }

  if y == sample_size - 1 {
    if err_map[y][x] == std::u32::MAX {
      //println!("{} {} {} {:?}", sample_size, y, x, err_map);
    }
    path_map[y][x] = PathPoint {
      error: err_map[y][x] + path_map[y][x + 1].error,
      to_parent: Move::Horizontal,
    };
    return;
  }

  if x == sample_size - 1 {
    if err_map[y][x] == std::u32::MAX {
      //println!("{} {} {} {:?}", sample_size, y, x, err_map);
    }
    path_map[y][x] = PathPoint {
      error: err_map[y][x] + path_map[y + 1][x].error,
      to_parent: Move::Vertical,
    };
    return;
  }

  let top_err = match err_map[y + 1][x] == std::u32::MAX {
    true => std::u32::MAX,
    false => path_map[y + 1][x].error,
  };

  let right_err = match err_map[y][x + 1] == std::u32::MAX {
    true => std::u32::MAX,
    false => path_map[y][x + 1].error,
  };

  let diag_err = match err_map[y + 1][x + 1] == std::u32::MAX {
    true => std::u32::MAX,
    false => path_map[y + 1][x + 1].error,
  };

  let min_err = std::cmp::min(top_err, std::cmp::min(right_err, diag_err));

  if diag_err == min_err {
    if err_map[y][x] == std::u32::MAX {
      //println!("{} {} {} {:?}", sample_size, y, x, err_map);
    }
    path_map[y][x] = PathPoint {
      error: err_map[y][x] + diag_err,
      to_parent: Move::Diagonal,
    };
    return;
  }

  if top_err == min_err {
    if err_map[y][x] == std::u32::MAX {
      //println!("{} {} {} {:?}", sample_size, y, x, err_map);
    }
    path_map[y][x] = PathPoint {
      error: err_map[y][x] + top_err,
      to_parent: Move::Vertical,
    };
    return;
  }

  if right_err == min_err {
    if err_map[y][x] == std::u32::MAX {
      //println!("{} {} {} {:?}", sample_size, y, x, err_map);
    }
    path_map[y][x] = PathPoint {
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
  downsample_path: &Option<Path>,
) -> [[u32; SIG_SIZE]; SIG_SIZE] {
  // Allocate space for the error map
  let mut err_map: [[u32; SIG_SIZE]; SIG_SIZE] =
    unsafe { std::mem::MaybeUninit::uninit().assume_init() };
  //[[0u32; SIG_SIZE]; SIG_SIZE];

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
      set_top_right_bounds(1, 1, &mut err_map);

      // Now follow the downsample path through the map
      // (from the beginning of the signals), only
      // calculating error values for adjacent cells.

      // Coordinates of the current path step on the upsample
      let mut x = 1;
      let mut y = 1;
      let mut last_move: Option<Move> = None;

      for path_move in dp.iter() {
        match path_move.to_parent {
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
            }

            // Set 4 candidate blocks
            calc_err_cell(y - 1, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y - 1, x, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x - 1, &sig_y, &sig_x, &mut err_map);
            calc_err_cell(y, x, &sig_y, &sig_x, &mut err_map);
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

        set_top_right_bounds(y, x, &mut err_map);
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

fn set_top_right_bounds(y: usize, x: usize, err_map: &mut [[u32; SIG_SIZE]; SIG_SIZE]) {
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
