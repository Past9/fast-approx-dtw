use libm::sqrtf;

#[inline]
pub fn dist_u8(y: &u8, x: &u8) -> u32 {
  (*y as i16 - *x as i16).abs() as u32
}

#[inline]
pub fn euclidean_u8<const N: usize>(y: &[u8; N], x: &[u8; N]) -> u32 {
  let mut accum: u32 = 0;
  for i in 0..N {
    accum += dist_u8(&y[i], &x[i]);
  }
  sqrtf(accum as f32) as u32
}
