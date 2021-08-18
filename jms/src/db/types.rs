use std::{fmt::Display, mem};

use uuid::Uuid;

pub trait Key: Sized + std::fmt::Debug + AsRef<[u8]> + Clone {
  fn from_raw(r: &[u8]) -> Self;
  fn generate(db: &super::Store) -> Self;
}

impl Key for Integer {
  fn from_raw(r: &[u8]) -> Self {
    Integer::from(r)
  }

  fn generate(db: &super::Store) -> Self {
    Integer::from(db.generate_id().unwrap())
  }
}

impl Key for String {
  fn from_raw(r: &[u8]) -> Self {
    std::str::from_utf8(r).unwrap().to_string()
  }

  fn generate(_: &super::Store) -> Self {
    Uuid::new_v4().to_string()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Integer([u8; 16]);

impl From<u128> for Integer {
  fn from(i: u128) -> Integer {
    unsafe { Integer(mem::transmute(i.to_be())) }
  }
}

impl From<u64> for Integer {
  fn from(i: u64) -> Integer {
    let i = i as u128;
    i.into()
  }
}

impl From<u32> for Integer {
  fn from(i: u32) -> Integer {
    let i = i as u128;
    i.into()
  }
}

impl From<i32> for Integer {
  fn from(i: i32) -> Integer {
    let i = i as u128;
    i.into()
  }
}

impl From<usize> for Integer {
  fn from(i: usize) -> Integer {
    let i = i as u128;
    i.into()
  }
}

impl From<Integer> for u128 {
  #[cfg(target_endian = "big")]
  fn from(i: Integer) -> u128 {
    unsafe { mem::transmute(i.0) }
  }

  #[cfg(target_endian = "little")]
  fn from(i: Integer) -> u128 {
    u128::from_be(unsafe { mem::transmute(i.0) })
  }
}

impl From<Integer> for u64 {
  fn from(i: Integer) -> u64 {
    let i: u128 = i.into();
    i as u64
  }
}

impl From<Integer> for usize {
  fn from(i: Integer) -> usize {
    let i: u128 = i.into();
    i as usize
  }
}

impl AsRef<[u8]> for Integer {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

impl<'a> From<&'a [u8]> for Integer {
  fn from(buf: &'a [u8]) -> Integer {
    let mut dst = Integer::from(0u128);
    dst.0[..16].clone_from_slice(&buf[..16]);
    dst
  }
}

impl Display for Integer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let a: u128 = self.clone().into();
    write!(f, "{}", a)
  }
}