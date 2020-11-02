use crate::alloc::alloc;
use crate::stack_vec::StackVec;
use crate::SIG_SIZE;

#[derive(Debug)]
pub struct Downsample<SampleType> {
  pub signal: [SampleType; SIG_SIZE],
  pub len: usize,
}
impl<SampleType> Downsample<SampleType> {
  #[inline]
  pub fn create_one<'a>(
    signal: &[SampleType; SIG_SIZE],
    len: usize,
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  ) -> Option<Downsample<SampleType>> {
    if len < 4 || len % 2 != 0 {
      return None;
    }

    let mut ds_signal: [SampleType; SIG_SIZE] = alloc(false);

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
    signal: &[SampleType; SIG_SIZE],
    downsample_fn: fn(&SampleType, &SampleType) -> SampleType,
  ) -> StackVec<Downsample<SampleType>, MAX_DOWNSAMPLES> {
    let mut downsamples = StackVec::<Downsample<SampleType>, MAX_DOWNSAMPLES>::empty(false);

    loop {
      let (sig, len) = match downsamples.len() {
        0 => (signal, SIG_SIZE),
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
