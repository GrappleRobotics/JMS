use std::sync::{Arc, Mutex};

pub use redis::*;
pub use redis_macros::Json;

// We had the opportunity to make this async, but given the sub-millisecond latency of Redis, 
// the async overhead is most likely higher than the performance gains that we stand to get. 

pub struct KVConnection {
  client: Arc<redis::Client>,
  redis: std::sync::Mutex<redis::Connection>
}

impl KVConnection {
  pub fn new() -> anyhow::Result<Self> {
    let redis_uri = std::env::var("REDIS_URI").unwrap_or("redis://localhost:6379/0".to_owned());
    let redis_client = redis::Client::open(redis_uri.clone())?;
    let redis_connection = redis_client.get_connection()?;

    Ok(Self {
      client: Arc::new(redis_client),
      redis: std::sync::Mutex::new(redis_connection)
    })
  }

  pub fn clone(&self) -> anyhow::Result<Self> {
    Ok(Self {
      client: self.client.clone(),
      redis: Mutex::new(self.client.get_connection()?)
    })
  }

  pub fn expire(&self, key: &str, seconds: usize) -> anyhow::Result<()> {
    self.redis.lock().unwrap().expire(key, seconds)?;
    Ok(())
  }

  pub fn json_set<V: serde::Serialize>(&self, key: &str, path: &str, value: &V) -> anyhow::Result<()> {
    self.redis.lock().unwrap().json_set(key, path, value)?;
    Ok(())
  }

  pub fn json_get<V: serde::de::DeserializeOwned>(&self, key: &str, path: &str) -> anyhow::Result<V> {
    let Json(us): Json<V> = self.redis.lock().unwrap().json_get(key, path)?;
    Ok(us)
  }

  pub fn hset<V: ToRedisArgs>(&self, key: &str, field: &str, value: V) -> anyhow::Result<()> {
    self.redis.lock().unwrap().hset(key, field, value)?;
    Ok(())
  }

  pub fn hget<RV: FromRedisValue>(&self, key: &str, field: &str) -> anyhow::Result<RV> {
    Ok(self.redis.lock().unwrap().hget(key, field)?)
  }

  pub fn set<V: ToRedisArgs>(&self, key: &str, value: V) -> anyhow::Result<()> {
    self.redis.lock().unwrap().set(key, value)?;
    Ok(())
  }

  pub fn setnx<V: ToRedisArgs>(&self, key: &str, value: V) -> anyhow::Result<()> {
    self.redis.lock().unwrap().set_nx(key, value)?;
    Ok(())
  }

  pub fn get<RV: FromRedisValue>(&self, key: &str) -> anyhow::Result<RV> {
    Ok(self.redis.lock().unwrap().get(key)?)
  }

  pub fn exists(&self, key: &str) -> anyhow::Result<bool> {
    Ok(self.redis.lock().unwrap().exists(key)?)
  }

  pub fn del(&self, key: &str) -> anyhow::Result<()> {
    self.redis.lock().unwrap().del(key)?;
    Ok(())
  }

  pub fn keys(&self, pattern: &str) -> anyhow::Result<Vec<String>> {
    Ok(self.redis.lock().unwrap().keys(pattern)?)
  }
}