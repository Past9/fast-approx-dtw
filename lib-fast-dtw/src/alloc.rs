#[inline]
pub fn alloc<T>(zero_mem: bool) -> T {
  match zero_mem {
    true => unsafe { core::mem::zeroed() },
    false => unsafe { core::mem::MaybeUninit::uninit().assume_init() },
  }
}
