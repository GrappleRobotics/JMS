mod schedule;

use jms_base::kv::{KVConnection, self};
use jms_core_lib::{db::{Table, self}, models::{Team, ScheduleBlock}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kv = kv::KVConnection::new().await?;

    let block = ScheduleBlock {
        id: db::generate_id(),
        name: "Some Block".into(),
        block_type: jms_core_lib::models::ScheduleBlockType::Qualification,
        start_time: chrono::Local::now(),
        end_time: chrono::Local::now(),
        cycle_time: chrono::Duration::milliseconds(2000).into()
    };

    block.insert(&kv).await?;

    println!("{:?}", ScheduleBlock::all(&kv).await?);

    Ok(())
}
