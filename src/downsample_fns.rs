use crate::alloc::alloc;

#[inline]
pub fn mean_u8(s1: &u8, s2: &u8) -> u8 {
  ((*s1 as u16 + *s2 as u16) / 2) as u8
  //*s1 / 2 + *s2 / 2
}

#[inline]
pub fn mean<const N: usize>(s1: &[u8; N], s2: &[u8; N]) -> [u8; N] {
  let mut mean: [u8; N] = alloc(false);

  for i in 0..N {
    mean[i] = mean_u8(&s1[i], &s2[i]);
  }

  mean
}
