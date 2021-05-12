use crate::log_expect;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use log::info;
use std::env;
// use diesel::sqlite::SqliteConnection;
use diesel::pg::PgConnection;
use lazy_static::lazy_static;

embed_migrations!("migrations");

pub type ConnectionT = PgConnection;
pub type DbPool = Pool<ConnectionManager<ConnectionT>>;
pub type DbPooledConnection = PooledConnection<ConnectionManager<ConnectionT>>;

fn pool() -> DbPool {
  lazy_static! {
    static ref POOL: DbPool = {
      info!("DB Pool Starting...");
      let uri = log_expect!(env::var("DATABASE_URL"), "DATABASE_URL is not set: {}");
      let mgr = ConnectionManager::<ConnectionT>::new(uri);
      let p = log_expect!(Pool::builder().build(mgr), "Could not start DB connection pool! {}");
      info!("DB Pool Ready!");
      p
    };
  }
  POOL.clone()
}

pub fn connection() -> DbPooledConnection {
  log_expect!(
    pool().get(),
    "Could not get a DB connection from the connection pool! {}"
  )
}
