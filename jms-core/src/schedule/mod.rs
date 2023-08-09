use jms_base::{kv::KVConnection, mq::MessageQueueChannel};
use jms_core_lib::schedule::generators::{MatchGeneratorRPC, QualsMatchGeneratorParams, MATCH_GENERATOR_JOB_KEY};
use log::{info, error};

use self::{quals::QualsMatchGenerator, playoffs::PlayoffMatchGenerator};

pub mod bracket;
pub mod quals_randomiser;
pub mod quals;
pub mod playoffs;

pub struct GeneratorService {
  pub kv: KVConnection,
  pub mq: MessageQueueChannel
}

#[async_trait::async_trait]
impl MatchGeneratorRPC for GeneratorService {
  fn mq(&self) -> &MessageQueueChannel { &self.mq }

  async fn start_qual_gen(&mut self, params: QualsMatchGeneratorParams) -> Result<(), String> {
    let kv = self.kv.clone().map_err(|e| e.to_string())?;

    info!("Starting Quals Generator...");
    if kv.get::<bool>(MATCH_GENERATOR_JOB_KEY).map_err(|e| e.to_string())? {
      return Err("Quals Generator is already running!".to_owned())
    }
    
    kv.set(MATCH_GENERATOR_JOB_KEY, true).map_err(|e| e.to_string())?;
    kv.expire(MATCH_GENERATOR_JOB_KEY, 20*60).map_err(|e| e.to_string())?;    // 20 mins, just incase we panic

    tokio::task::spawn(async move {
      let gen = QualsMatchGenerator::new();
      match gen.generate(params, &kv) {
        Ok(()) => info!("Quals Generated!"),
        Err(e) => error!("Error in Quals Generation: {}", e)
      }
      kv.del(MATCH_GENERATOR_JOB_KEY).ok();
    });
    Ok(())
  }

  async fn reset_playoffs(&mut self) -> Result<(), String> {
    PlayoffMatchGenerator::reset(&self.kv).map_err(|e| e.to_string())
  }

  async fn update_playoffs(&mut self) -> Result<(), String> {
    PlayoffMatchGenerator::update(&self.kv).map_err(|e| e.to_string())
  }
}

impl GeneratorService {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut rpc = self.rpc_handle().await?;
    loop {
      self.rpc_process(rpc.next().await).await?;
    }
  }
}