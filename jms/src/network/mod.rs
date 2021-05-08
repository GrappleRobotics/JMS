pub mod onboard;

use crate::arena::AllianceStation;

pub type NetworkError = std::io::Error;
pub type NetworkResult<T> = std::result::Result<T, NetworkError>; // TODO: Update to the new type
pub type NetworkFuture<T> = dyn futures::Future<Output=NetworkResult<T>>;

pub trait NetworkProvider {
  fn configure_admin(&mut self) -> NetworkResult<()>;

  fn configure_alliances(
    &mut self,
    stations: &mut dyn Iterator<Item = &AllianceStation>,
    force_reload: bool,
  ) -> NetworkResult<()>;
}
