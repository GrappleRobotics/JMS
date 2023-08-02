mod schedule;

use jms_base::kv::{KVConnection, self};
use jms_core_lib::{db::{Table, self}, models::{Team, ScheduleBlock}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kv = kv::KVConnection::new()?;

    Ok(())
}
