// Based on rust-kv: https://github.com/zshipko/rust-kv
mod store;
mod table;
mod types;
mod bindings;

pub use store::*;
pub use table::*;
pub use types::*;
pub use bindings::*;

pub type Result<T> = anyhow::Result<T>;

use lazy_static::lazy_static;

lazy_static! {
  static ref STORE: Store = {
    let cfg = sled::Config::new()
      .path("event.kvdb".to_owned())
      .flush_every_ms(Some(1000));

    Store::new(cfg).expect("Sled Store Could Not Open!")
  };
}

#[allow(dead_code)]
pub fn database() -> &'static Store {
  &STORE
}
