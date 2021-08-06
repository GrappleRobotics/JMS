pub mod bufutil;
pub mod danger;
pub mod service_configs;
pub mod templates;
pub mod ssh;

pub fn saturating_offset(base: usize, delta: isize) -> usize {
  if delta < 0 {
    base.checked_sub(delta.wrapping_abs() as usize).unwrap_or(0)
  } else {
    base.checked_add(delta as usize).unwrap_or(0)
  }
}