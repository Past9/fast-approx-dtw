use crate::NUM_DOWNSAMPLES;
use crate::SIG_SIZE;

#[derive(Debug, Copy, Clone)]
pub struct Downsample {
  pub signal: [u8; SIG_SIZE],
  pub len: usize,
}
impl Downsample {
  #[inline]
  pub fn create_one(signal: &[u8; SIG_SIZE], len: usize) -> Option<Downsample> {
    if len % 2 != 0 {
      return None;
    }

    let mut downsample = Downsample {
      signal: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
      len: len / 2,
    };

    for t in 0..downsample.len {
      downsample.signal[t] = signal[t * 2] / 2 + signal[t * 2 + 1] / 2
    }

    Some(downsample)
  }

  #[inline]
  pub fn create_all(signal: &[u8; SIG_SIZE]) -> [Downsample; NUM_DOWNSAMPLES] {
    let mut downsamples: [Downsample; NUM_DOWNSAMPLES] =
      unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    let mut last_downsample = Downsample {
      signal: *signal,
      len: SIG_SIZE,
    };

    for i in 0..NUM_DOWNSAMPLES {
      match Downsample::create_one(&last_downsample.signal, last_downsample.len) {
        Some(ds) => {
          downsamples[NUM_DOWNSAMPLES - i - 1] = last_downsample;
          last_downsample = ds;
        }
        None => {
          break;
        }
      }
    }

    downsamples
  }
}
