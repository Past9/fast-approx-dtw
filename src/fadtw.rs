use crate::alloc::alloc;
use crate::downsample::Downsample;
use crate::path::*;
use crate::SIG_SIZE;

const MAX_DOWNSAMPLES: usize = 16;

pub struct DtwSolver<'a, SampleType, const MaxPathLen: usize> {
  sig_y: &'a [SampleType; SIG_SIZE],
  sig_x: &'a [SampleType; SIG_SIZE],
  signal_size: usize,
  downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  err_fn: fn(&SampleType, &SampleType) -> u32,
  err_map: [[u32; SIG_SIZE]; SIG_SIZE],
  path_map: [[PathPoint; SIG_SIZE]; SIG_SIZE],
  downsample_limit: Option<usize>,
}
impl<'a, SampleType, const MaxPathLen: usize> DtwSolver<'a, SampleType, MaxPathLen> {
  pub fn new(
    sig_y: &'a [SampleType; SIG_SIZE],
    sig_x: &'a [SampleType; SIG_SIZE],
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
    err_fn: fn(&SampleType, &SampleType) -> u32,
  ) -> DtwSolver<'a, SampleType, MaxPathLen> {
    DtwSolver {
      sig_y,
      sig_x,
      signal_size: SIG_SIZE,
      downsample_fn,
      err_fn,
      err_map: alloc(false),
      path_map: alloc(false),
      downsample_limit: None,
    }
  }

  pub fn limit_downsamples(&mut self, max_downsamples: usize) -> &mut Self {
    self.downsample_limit = Some(max_downsamples);
    self
  }

  fn use_signal_size(&mut self, size: usize) {
    self.signal_size = size;
  }

  fn guided_solve(&mut self, downsample_path: &Option<Path<MaxPathLen>>) -> Path<MaxPathLen> {
    self.gen_err_map(&downsample_path);
    self.map_paths(&downsample_path);
    self.get_best_path()
  }

  #[inline]
  pub fn solve(&mut self) -> Path<MaxPathLen> {
    let downsamples_y = Downsample::create_all::<MAX_DOWNSAMPLES>(self.sig_y, self.downsample_fn);
    let downsamples_x = Downsample::create_all::<MAX_DOWNSAMPLES>(self.sig_x, self.downsample_fn);
    let mut last_downsample_path = None;

    for mi in 0..downsamples_y.len() {
      let i = downsamples_y.len() - mi - 1;
      let mut solver = DtwSolver::<SampleType, MaxPathLen>::new(
        &downsamples_y[i].signal,
        &downsamples_x[i].signal,
        self.downsample_fn,
        self.err_fn,
      );
      solver.use_signal_size(downsamples_y[i].len);
      last_downsample_path = Some(solver.guided_solve(&last_downsample_path));
    }

    self.guided_solve(&last_downsample_path)
  }

  #[inline]
  pub fn gen_err_map(&mut self, downsample_path: &Option<Path<MaxPathLen>>) {
    // If we're building a subsample map, then there are uninitialized
    // values to the top and right of the top-right cell. We need to set
    // them to infinity so they don't mess up our min calculations later.
    if self.signal_size < SIG_SIZE {
      self.err_map[self.signal_size][self.signal_size - 2] = u32::MAX;
      self.err_map[self.signal_size][self.signal_size - 1] = u32::MAX;
      self.err_map[self.signal_size][self.signal_size] = u32::MAX;
      self.err_map[self.signal_size - 1][self.signal_size] = u32::MAX;
      self.err_map[self.signal_size - 2][self.signal_size] = u32::MAX;
    }

    match downsample_path {
      Some(ref dp) => {
        // We always have to calculate errors for the 4 cells near the
        // origin
        self.calc_err_cell(0, 0); // Corner
        self.calc_err_cell(0, 1); // Top
        self.calc_err_cell(1, 0); // Right
        self.calc_err_cell(1, 1); // Top-right
        self.set_top_right_bounds(1, 1);

        // Now follow the downsample path through the map
        // (from the beginning of the signals), only
        // calculating error values for adjacent cells.

        // Coordinates of the current path step on the upsample
        let mut x = 1;
        let mut y = 1;
        let last_move: Option<Move> = None;

        for path_move in dp.iter() {
          match path_move.to_parent {
            Move::Vertical => {
              // Going up
              y += 2;

              // Set 3 boundary cells to "infinity",
              // unless we're by the left edge
              if x > 1 {
                self.err_map[y][x - 2] = u32::MAX;
                self.err_map[y - 1][x - 2] = u32::MAX;

                // only set this one if we didn't move
                // right just before this, because if we did,
                // then we'll overwrite a previously calculated
                // cell.
                match last_move {
                  Some(Move::Horizontal) => {
                    self.err_map[y - 2][x - 2] = u32::MAX;
                  }
                  _ => {}
                };
              }

              // Set 4 candidate blocks
              self.calc_err_cell(y - 1, x - 1);
              self.calc_err_cell(y - 1, x);
              self.calc_err_cell(y, x - 1);
              self.calc_err_cell(y, x);
            }
            Move::Horizontal => {
              // Going right
              x += 2;

              // Set 3 boundary cells to "infinity",
              // unless we're by the bottom edge
              if y > 1 {
                self.err_map[y - 2][x] = u32::MAX;
                self.err_map[y - 2][x - 1] = u32::MAX;

                // only set this one if we didn't move
                // up just before this, because if we did,
                // then we'll overwrite a previously calculated
                // cell.
                match last_move {
                  Some(Move::Vertical) => {
                    self.err_map[y - 2][x - 2] = u32::MAX;
                  }
                  _ => {}
                };
              }

              // Set 4 candidate blocks
              self.calc_err_cell(y - 1, x - 1);
              self.calc_err_cell(y - 1, x);
              self.calc_err_cell(y, x - 1);
              self.calc_err_cell(y, x);
            }
            Move::Diagonal => {
              // Going up and right
              y += 2;
              x += 2;

              // Set 4 boundary cells to "infinity"
              self.err_map[y][x - 2] = u32::MAX;
              self.err_map[y - 1][x - 3] = u32::MAX;
              self.err_map[y - 2][x] = u32::MAX;
              self.err_map[y - 3][x - 1] = u32::MAX;

              // Set 6 candidate blocks
              self.calc_err_cell(y - 2, x - 1);
              self.calc_err_cell(y - 1, x - 2);
              self.calc_err_cell(y - 1, x - 1);
              self.calc_err_cell(y - 1, x);
              self.calc_err_cell(y, x - 1);
              self.calc_err_cell(y, x);
            }
            Move::Stop => panic!("Invalid move"), // This variant doesn't apply here
          };

          self.set_top_right_bounds(y, x);
        }
      }
      None => {
        // If we weren't given a downsample path, we fill out the
        // error map completely.
        for y in 0..self.signal_size {
          for x in 0..self.signal_size {
            self.calc_err_cell(y, x);
          }
        }
      }
    }
  }

  #[inline]
  pub fn calc_err_cell(&mut self, y: usize, x: usize) {
    let err = (self.err_fn)(&self.sig_y[y], &self.sig_x[x]);
    let left: u32 = match x == 0 {
      true => u32::MAX,
      false => self.err_map[y][x - 1],
    };
    let down: u32 = match y == 0 {
      true => u32::MAX,
      false => self.err_map[y - 1][x],
    };
    let down_left: u32 = match y == 0 || x == 0 {
      true => u32::MAX,
      false => self.err_map[y - 1][x - 1],
    };

    let mut min = core::cmp::min(left, core::cmp::min(down, down_left));
    if min == u32::MAX {
      min = 0;
    }
    self.err_map[y][x] = err + min;
  }

  #[inline]
  fn set_top_right_bounds(&mut self, y: usize, x: usize) {
    // Set 5 boundary cells to the top and right
    if y < SIG_SIZE - 1 {
      self.err_map[y + 1][x - 1] = u32::MAX;
      self.err_map[y + 1][x] = u32::MAX;
    }

    if y < SIG_SIZE - 1 && x < SIG_SIZE - 1 {
      self.err_map[y + 1][x + 1] = u32::MAX;
    }

    if x < SIG_SIZE - 1 {
      self.err_map[y][x + 1] = u32::MAX;
      self.err_map[y - 1][x + 1] = u32::MAX;
    }
  }

  #[inline]
  pub fn map_paths(&mut self, downsample_path: &Option<Path<MaxPathLen>>) -> Path<MaxPathLen> {
    let mut path_map: [[PathPoint; SIG_SIZE]; SIG_SIZE] = alloc(false);

    match downsample_path {
      Some(dp) => {
        // If we have a downsample path, then we only calculate possible paths through
        // the cells adjacent to it.
        let mut y = self.signal_size - 1;
        let mut x = self.signal_size - 1;

        // Initialize the corner "Stop" cell and its 3 adjacent cells
        self.calc_path_cell(y, x);
        self.calc_path_cell(y, x - 1);
        self.calc_path_cell(y - 1, x);
        self.calc_path_cell(y - 1, x - 1);

        for path_move in dp.iter().rev() {
          match path_move.to_parent {
            Move::Vertical => {
              y -= 2;
              self.calc_path_cell(y, x);
              self.calc_path_cell(y, x - 1);
              self.calc_path_cell(y - 1, x);
              self.calc_path_cell(y - 1, x - 1);
            }
            Move::Horizontal => {
              x -= 2;
              self.calc_path_cell(y, x);
              self.calc_path_cell(y, x - 1);
              self.calc_path_cell(y - 1, x);
              self.calc_path_cell(y - 1, x - 1);
            }
            Move::Diagonal => {
              y -= 2;
              x -= 2;
              self.calc_path_cell(y + 1, x);
              self.calc_path_cell(y, x + 1);
              self.calc_path_cell(y, x);
              self.calc_path_cell(y, x - 1);
              self.calc_path_cell(y - 1, x);
              self.calc_path_cell(y - 1, x - 1);
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
        for my in 0..self.signal_size {
          let y = self.signal_size - my - 1;
          for mx in 0..self.signal_size {
            let x = self.signal_size - mx - 1;
            self.calc_path_cell(y, x);
          }
        }
      }
    };

    self.get_best_path()
  }

  #[inline]
  pub fn calc_path_cell(&mut self, y: usize, x: usize) {
    if y == self.signal_size - 1 && x == self.signal_size - 1 {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x],
        to_parent: Move::Stop,
      };
      return;
    }

    if y == self.signal_size - 1 {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x] + self.path_map[y][x + 1].error,
        to_parent: Move::Horizontal,
      };
      return;
    }

    if x == self.signal_size - 1 {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x] + self.path_map[y + 1][x].error,
        to_parent: Move::Vertical,
      };
      return;
    }

    let top_err = match self.err_map[y + 1][x] == u32::MAX {
      true => u32::MAX,
      false => self.path_map[y + 1][x].error,
    };

    let right_err = match self.err_map[y][x + 1] == u32::MAX {
      true => u32::MAX,
      false => self.path_map[y][x + 1].error,
    };

    let diag_err = match self.err_map[y + 1][x + 1] == u32::MAX {
      true => u32::MAX,
      false => self.path_map[y + 1][x + 1].error,
    };

    let min_err = core::cmp::min(top_err, core::cmp::min(right_err, diag_err));

    if diag_err == min_err {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x] + diag_err,
        to_parent: Move::Diagonal,
      };
      return;
    }

    if top_err == min_err {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x] + top_err,
        to_parent: Move::Vertical,
      };
      return;
    }

    if right_err == min_err {
      self.path_map[y][x] = PathPoint {
        error: self.err_map[y][x] + right_err,
        to_parent: Move::Horizontal,
      };
      return;
    }
  }

  #[inline]
  pub fn get_best_path(&self) -> Path<MaxPathLen> {
    let mut y = 0;
    let mut x = 0;
    let mut current_cell = self.path_map[y][x];
    let mut path = Path::empty(false);
    //let mut len = 0;
    //let mut moves: [PathPoint; MAX_PATH_LEN] = alloc(false);

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

      path.push(current_cell);

      current_cell = self.path_map[y][x];
      if current_cell.to_parent == Move::Stop {
        break;
      }
    }

    path
  }
}

/*
#[inline]
pub fn fast_approx_dtw<SampleType: Clone, const MaxPathLen: usize>(
  sig_y: &[SampleType; SIG_SIZE],
  sig_x: &[SampleType; SIG_SIZE],
  downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  err_fn: fn(&SampleType, &SampleType) -> u32,
) -> Path<MaxPathLen> {
  let downsamples_y = Downsample::create_all::<MAX_DOWNSAMPLES>(sig_y.clone(), downsample_fn);
  let downsamples_x = Downsample::create_all::<MAX_DOWNSAMPLES>(sig_x.clone(), downsample_fn);

  let mut last_downsample_path: Option<Path<MaxPathLen>> = None;

  for mi in 0..downsamples_y.len() {
    let i = downsamples_y.len() - mi - 1;
    let err_map = gen_err_map(
      &downsamples_y[i].signal,
      &downsamples_x[i].signal,
      downsamples_y[i].len,
      &last_downsample_path,
      err_fn,
    );
    last_downsample_path = Some(map_paths(
      &err_map,
      downsamples_y[i].len,
      &last_downsample_path,
    ));
  }

  last_downsample_path.unwrap()
}

*/
