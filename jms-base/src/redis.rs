pub use redis::*;

pub async fn redis_connect() -> anyhow::Result<(redis::Client, redis::aio::Connection)> {
  let redis_uri = std::env::var("REDIS_URI").unwrap_or("redis://redis:6379".to_owned());
  let redis_client = redis::Client::open(redis_uri)?;
  let redis_connection = redis_client.get_async_connection().await?;
  Ok((redis_client, redis_connection))
}