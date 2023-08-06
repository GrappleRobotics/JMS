use std::time::Duration;

use jms_base::kv;
use jms_core_lib::{scoring::scores::{MatchScoreSnapshot, MatchScore, ScoreUpdateData}, db::Singleton, models::{MaybeToken, Alliance, Permission}};
use uuid::Uuid;

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait ScoringWebsocket {
  #[publish]
  async fn current(&self, ctx: &WebsocketContext) -> anyhow::Result<MatchScoreSnapshot> {
    Ok(MatchScore::get(&ctx.kv)?.into())
  }

  async fn do_score_update(kv: &kv::KVConnection, update: ScoreUpdateData) -> anyhow::Result<MatchScoreSnapshot> {
    let mut live_score = MatchScore::get(kv)?;
    match update.alliance {
      Alliance::Red => live_score.red.update(update.update),
      Alliance::Blue => live_score.blue.update(update.update)
    }
    live_score.update(kv)?;
    Ok(live_score.into())
  }

  // TODO: Use something like immutability-helper but in Rust so we're not taking the entire object, as it may fail to register
  // some calls if two score_updates get queued at the same time. This function should be atomic.
  // TODO: Implement redlock for this.
  #[endpoint]
  async fn score_update(&self, ctx: &WebsocketContext, token: &MaybeToken, update: ScoreUpdateData) -> anyhow::Result<MatchScoreSnapshot> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::Scoring])?;
    
    let my_id = Uuid::new_v4().to_string();

    loop {
      ctx.kv.setnx("__score_update_lock", &my_id)?;
      if ctx.kv.get::<String>("__score_update_lock")? == my_id {
        break;
      }
      tokio::time::sleep(Duration::from_millis(1)).await;
    }

    let r = Self::do_score_update(&ctx.kv, update).await;
    ctx.kv.del("__score_update_lock")?;
    r
  }
}