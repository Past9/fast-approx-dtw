use crate::alloc::alloc;
use crate::stack_vec::StackVec;

#[derive(Debug)]
pub struct Downsample<SampleType, const SIGNAL_SIZE: usize> {
  pub signal: [SampleType; SIGNAL_SIZE],
  pub len: usize,
}
impl<SampleType, const SIGNAL_SIZE: usize> Downsample<SampleType, SIGNAL_SIZE> {
  #[inline]
  pub fn create_one<'a>(
    signal: &[SampleType; SIGNAL_SIZE],
    len: usize,
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  ) -> Option<Downsample<SampleType, SIGNAL_SIZE>> {
    if len < 4 || len % 2 != 0 {
      return None;
    }

    let mut ds_signal: [SampleType; SIGNAL_SIZE] = alloc(false);

    for t in 0..(len / 2) {
      ds_signal[t] = downsample_fn(&signal[t * 2], &signal[t * 2 + 1]);
    }

    Some(Downsample {
      signal: ds_signal,
      len: len / 2,
    })
  }

  #[inline]
  pub fn create_all<const MAX_DOWNSAMPLES: usize>(
    signal: &[SampleType; SIGNAL_SIZE],
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
    downsample_limit: Option<usize>,
  ) -> StackVec<Downsample<SampleType, SIGNAL_SIZE>, MAX_DOWNSAMPLES> {
    let mut downsamples =
      StackVec::<Downsample<SampleType, SIGNAL_SIZE>, MAX_DOWNSAMPLES>::empty(false);

    let ds_limit = match downsample_limit {
      Some(limit) => core::cmp::min(limit, MAX_DOWNSAMPLES),
      None => MAX_DOWNSAMPLES,
    };

    for _ in 0..ds_limit {
      let (sig, len) = match downsamples.len() {
        0 => (signal, SIGNAL_SIZE),
        _ => (
          &downsamples[downsamples.len() - 1].signal,
          downsamples[downsamples.len() - 1].len,
        ),
      };

      match Downsample::create_one(sig, len, downsample_fn) {
        Some(ds) => {
          downsamples.push(ds);
        }
        None => {
          break;
        }
      };
    }

    downsamples
  }
}
