use std::sync::Arc;

pub use redis::*;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct KVStore {
  redis: Arc<RwLock<redis::aio::Connection>>
}

impl KVStore {
  pub async fn new() -> anyhow::Result<Self> {
    let redis_uri = std::env::var("REDIS_URI").unwrap_or("redis://redis:6379".to_owned());
    let redis_client = redis::Client::open(redis_uri)?;
    let redis_connection = redis_client.get_async_connection().await?;

    Ok(Self {
      redis: Arc::new(RwLock::new(redis_connection))
    })
  }

  pub async fn json_set<V: serde::Serialize + Send + Sync>(&self, key: &str, path: &str, value: &V) -> anyhow::Result<()> {
    self.redis.write().await.json_set(key, path, value).await?;
    Ok(())
  }

  pub async fn json_get<RV: FromRedisValue>(&self, key: &str, path: &str) -> anyhow::Result<RV> {
    Ok(self.redis.write().await.json_get(key, path).await?)
  }

  pub async fn hset<V: ToRedisArgs + Send + Sync>(&self, key: &str, field: &str, value: V) -> anyhow::Result<()> {
    self.redis.write().await.hset(key, field, value).await?;
    Ok(())
  }

  pub async fn hget<RV: FromRedisValue>(&self, key: &str, field: &str) -> anyhow::Result<RV> {
    Ok(self.redis.write().await.hget(key, field).await?)
  }

  pub async fn del(&self, key: &str) -> anyhow::Result<()> {
    self.redis.write().await.del(key).await?;
    Ok(())
  }
}