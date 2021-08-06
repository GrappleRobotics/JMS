pub mod onboard;
pub mod radio;

use crate::arena::AllianceStation;

use async_trait::async_trait;

pub type NetworkResult<T> = anyhow::Result<T>;

#[async_trait]
pub trait NetworkProvider {
  async fn configure(&self, stations: &[AllianceStation], force_reload: bool) -> NetworkResult<()>;
}
