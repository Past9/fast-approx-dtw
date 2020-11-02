use libm::sqrtf;

#[inline]
pub fn dist(y: &f32, x: &f32) -> f32 {
  libm::fabsf(*y - *x)
}

#[inline]
pub fn euclidean<const N: usize>(y: &[f32; N], x: &[f32; N]) -> f32 {
  let mut accum: f32 = 0f32;
  for i in 0..N {
    accum += dist(&y[i], &x[i]);
  }
  sqrtf(accum as f32)
}
