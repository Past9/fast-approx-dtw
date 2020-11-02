pub fn alloc<T>(zero_mem: bool) -> T {
  match zero_mem {
    true => unsafe { std::mem::zeroed() },
    false => unsafe { std::mem::MaybeUninit::uninit().assume_init() },
  }
}
