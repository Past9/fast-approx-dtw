use core::cmp;

use crate::alloc::alloc;
use crate::downsample::Downsample;
use crate::path::*;

const MAX_DOWNSAMPLES: usize = 16;
const INFINITY: f32 = core::f32::MAX;

pub struct DtwSolver<'a, SampleType, const SIGNAL_SIZE: usize, const MAX_PATH_LEN: usize> {
  sig_y: &'a [SampleType; SIGNAL_SIZE],
  sig_x: &'a [SampleType; SIGNAL_SIZE],
  signal_size: usize,
  downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  loss_fn: fn(&SampleType, &SampleType) -> f32,
  loss_map: [[f32; SIGNAL_SIZE]; SIGNAL_SIZE],
  path_map: [[PathPoint; SIGNAL_SIZE]; SIGNAL_SIZE],
  downsample_limit: Option<usize>,
}
impl<'a, SampleType, const SIGNAL_SIZE: usize, const MAX_PATH_LEN: usize>
  DtwSolver<'a, SampleType, SIGNAL_SIZE, MAX_PATH_LEN>
{
  pub fn new(
    sig_y: &'a [SampleType; SIGNAL_SIZE],
    sig_x: &'a [SampleType; SIGNAL_SIZE],
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
    loss_fn: fn(&SampleType, &SampleType) -> f32,
  ) -> DtwSolver<'a, SampleType, SIGNAL_SIZE, MAX_PATH_LEN> {
    DtwSolver {
      sig_y,
      sig_x,
      signal_size: SIGNAL_SIZE,
      downsample_fn,
      loss_fn,
      loss_map: alloc(false),
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

  fn guided_solve(&mut self, downsample_path: &Option<Path<MAX_PATH_LEN>>) -> Path<MAX_PATH_LEN> {
    self.map_losses(&downsample_path);
    self.map_paths(&downsample_path);
    self.get_best_path()
  }

  #[inline]
  pub fn solve(&mut self) -> Path<MAX_PATH_LEN> {
    let downsamples_y = Downsample::create_all::<MAX_DOWNSAMPLES>(
      self.sig_y,
      self.downsample_fn,
      self.downsample_limit,
    );
    let downsamples_x = Downsample::create_all::<MAX_DOWNSAMPLES>(
      self.sig_x,
      self.downsample_fn,
      self.downsample_limit,
    );
    let mut last_downsample_path = None;

    for mi in 0..downsamples_y.len() {
      let i = downsamples_y.len() - mi - 1;
      let mut solver = DtwSolver::<SampleType, SIGNAL_SIZE, MAX_PATH_LEN>::new(
        &downsamples_y[i].signal,
        &downsamples_x[i].signal,
        self.downsample_fn,
        self.loss_fn,
      );
      solver.use_signal_size(downsamples_y[i].len);
      last_downsample_path = Some(solver.guided_solve(&last_downsample_path));
    }

    self.guided_solve(&last_downsample_path)
  }

  #[inline]
  pub fn map_losses(&mut self, downsample_path: &Option<Path<MAX_PATH_LEN>>) {
    // If we're building a subsample map, then there are uninitialized
    // values to the top and right of the top-right cell. We need to set
    // them to infinity so they don't mess up our min calculations later.
    if self.signal_size < SIGNAL_SIZE {
      self.loss_map[self.signal_size][self.signal_size - 2] = INFINITY;
      self.loss_map[self.signal_size][self.signal_size - 1] = INFINITY;
      self.loss_map[self.signal_size][self.signal_size] = INFINITY;
      self.loss_map[self.signal_size - 1][self.signal_size] = INFINITY;
      self.loss_map[self.signal_size - 2][self.signal_size] = INFINITY;
    }

    match downsample_path {
      Some(ref dp) => {
        // We always have to calculate losses for the 4 cells near the
        // origin
        self.calc_loss_cell(0, 0); // Corner
        self.calc_loss_cell(0, 1); // Top
        self.calc_loss_cell(1, 0); // Right
        self.calc_loss_cell(1, 1); // Top-right
        self.set_top_right_bounds(1, 1);

        // Now follow the downsample path through the map
        // (from the beginning of the signals), only
        // calculating loss values for adjacent cells.

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
                self.loss_map[y][x - 2] = INFINITY;
                self.loss_map[y - 1][x - 2] = INFINITY;

                // only set this one if we didn't move
                // right just before this, because if we did,
                // then we'll overwrite a previously calculated
                // cell.
                match last_move {
                  Some(Move::Horizontal) => {
                    self.loss_map[y - 2][x - 2] = INFINITY;
                  }
                  _ => {}
                };
              }

              // Set 4 candidate blocks
              self.calc_loss_cell(y - 1, x - 1);
              self.calc_loss_cell(y - 1, x);
              self.calc_loss_cell(y, x - 1);
              self.calc_loss_cell(y, x);
            }
            Move::Horizontal => {
              // Going right
              x += 2;

              // Set 3 boundary cells to "infinity",
              // unless we're by the bottom edge
              if y > 1 {
                self.loss_map[y - 2][x] = INFINITY;
                self.loss_map[y - 2][x - 1] = INFINITY;

                // only set this one if we didn't move
                // up just before this, because if we did,
                // then we'll overwrite a previously calculated
                // cell.
                match last_move {
                  Some(Move::Vertical) => {
                    self.loss_map[y - 2][x - 2] = INFINITY;
                  }
                  _ => {}
                };
              }

              // Set 4 candidate blocks
              self.calc_loss_cell(y - 1, x - 1);
              self.calc_loss_cell(y - 1, x);
              self.calc_loss_cell(y, x - 1);
              self.calc_loss_cell(y, x);
            }
            Move::Diagonal => {
              // Going up and right
              y += 2;
              x += 2;

              // Set 4 boundary cells to "infinity"
              self.loss_map[y][x - 2] = INFINITY;
              self.loss_map[y - 1][x - 3] = INFINITY;
              self.loss_map[y - 2][x] = INFINITY;
              self.loss_map[y - 3][x - 1] = INFINITY;

              // Set 6 candidate blocks
              self.calc_loss_cell(y - 2, x - 1);
              self.calc_loss_cell(y - 1, x - 2);
              self.calc_loss_cell(y - 1, x - 1);
              self.calc_loss_cell(y - 1, x);
              self.calc_loss_cell(y, x - 1);
              self.calc_loss_cell(y, x);
            }
            Move::Stop => panic!("Invalid move"), // This variant doesn't apply here
          };

          self.set_top_right_bounds(y, x);
        }
      }
      None => {
        // If we weren't given a downsample path, we fill out the
        // loss map completely.
        for y in 0..self.signal_size {
          for x in 0..self.signal_size {
            self.calc_loss_cell(y, x);
          }
        }
      }
    }
  }

  #[inline]
  pub fn calc_loss_cell(&mut self, y: usize, x: usize) {
    let loss = (self.loss_fn)(&self.sig_y[y], &self.sig_x[x]);
    let left = match x == 0 {
      true => INFINITY,
      false => self.loss_map[y][x - 1],
    };
    let down = match y == 0 {
      true => INFINITY,
      false => self.loss_map[y - 1][x],
    };
    let down_left = match y == 0 || x == 0 {
      true => INFINITY,
      false => self.loss_map[y - 1][x - 1],
    };

    let mut min = libm::fminf(left, libm::fminf(down, down_left));
    if min == INFINITY {
      min = 0f32;
    }
    self.loss_map[y][x] = loss + min;
  }

  #[inline]
  fn set_top_right_bounds(&mut self, y: usize, x: usize) {
    // Set 5 boundary cells to the top and right
    if y < SIGNAL_SIZE - 1 {
      self.loss_map[y + 1][x - 1] = INFINITY;
      self.loss_map[y + 1][x] = INFINITY;
    }

    if y < SIGNAL_SIZE - 1 && x < SIGNAL_SIZE - 1 {
      self.loss_map[y + 1][x + 1] = INFINITY;
    }

    if x < SIGNAL_SIZE - 1 {
      self.loss_map[y][x + 1] = INFINITY;
      self.loss_map[y - 1][x + 1] = INFINITY;
    }
  }

  #[inline]
  pub fn map_paths(&mut self, downsample_path: &Option<Path<MAX_PATH_LEN>>) -> Path<MAX_PATH_LEN> {
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
        loss: self.loss_map[y][x],
        to_parent: Move::Stop,
      };
      return;
    }

    if y == self.signal_size - 1 {
      self.path_map[y][x] = PathPoint {
        loss: self.loss_map[y][x] + self.path_map[y][x + 1].loss,
        to_parent: Move::Horizontal,
      };
      return;
    }

    if x == self.signal_size - 1 {
      self.path_map[y][x] = PathPoint {
        loss: self.loss_map[y][x] + self.path_map[y + 1][x].loss,
        to_parent: Move::Vertical,
      };
      return;
    }

    let vertical_loss = match self.loss_map[y + 1][x] == INFINITY {
      true => INFINITY,
      false => self.path_map[y + 1][x].loss,
    };

    let horizontal_loss = match self.loss_map[y][x + 1] == INFINITY {
      true => INFINITY,
      false => self.path_map[y][x + 1].loss,
    };

    let diag_loss = match self.loss_map[y + 1][x + 1] == INFINITY {
      true => INFINITY,
      false => self.path_map[y + 1][x + 1].loss,
    };

    let min_loss = libm::fminf(vertical_loss, libm::fminf(horizontal_loss, diag_loss));

    if diag_loss == min_loss {
      self.path_map[y][x] = PathPoint {
        loss: self.loss_map[y][x] + diag_loss,
        to_parent: Move::Diagonal,
      };
      return;
    }

    if vertical_loss == min_loss {
      self.path_map[y][x] = PathPoint {
        loss: self.loss_map[y][x] + vertical_loss,
        to_parent: Move::Vertical,
      };
      return;
    }

    if horizontal_loss == min_loss {
      self.path_map[y][x] = PathPoint {
        loss: self.loss_map[y][x] + horizontal_loss,
        to_parent: Move::Horizontal,
      };
      return;
    }
  }

  #[inline]
  pub fn get_best_path(&self) -> Path<MAX_PATH_LEN> {
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
