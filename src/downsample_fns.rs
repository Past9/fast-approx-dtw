use crate::alloc::alloc;

#[inline]
pub fn mean_u8(s1: &f32, s2: &f32) -> f32 {
  (*s1 + *s2) / 2f32
  //*s1 / 2 + *s2 / 2
}

#[inline]
pub fn mean<const N: usize>(s1: &[f32; N], s2: &[f32; N]) -> [f32; N] {
  let mut mean: [f32; N] = alloc(false);

  for i in 0..N {
    mean[i] = mean_u8(&s1[i], &s2[i]);
  }

  mean
}
