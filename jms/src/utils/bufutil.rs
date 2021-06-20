use std::io::{Error, ErrorKind};

use bytes::BytesMut;

pub fn utf8_str_with_len(buf: &mut BytesMut, len: usize) -> Result<String, Error> {
  let msg_buf = buf.split_to(len);
  let s = std::str::from_utf8(&msg_buf);
  match s {
    Ok(s) => Ok(s.to_owned()),
    Err(e) => Err(Error::new(ErrorKind::Other, format!("Utf8Error: {}", e)))
  }
}