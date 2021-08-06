pub mod onboard;
pub mod radio;

use crate::arena::AllianceStation;

use async_trait::async_trait;

pub type NetworkError = Box<dyn std::error::Error + Send + Sync>;
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

#[async_trait]
pub trait NetworkProvider {
  async fn configure(&self, stations: &[AllianceStation], force_reload: bool) -> NetworkResult<()>;
}
