pub trait Packable {
  fn pack(&self, buf: &mut dyn bytes::BufMut);
}

pub trait Unpackable {
  fn unpack(buf: &mut dyn bytes::Buf) -> Self;
}
