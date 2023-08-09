use std::time::Duration;

use jms_base::kv;
use jms_core_lib::{scoring::scores::{MatchScoreSnapshot, MatchScore, ScoreUpdateData, LiveScore}, db::{Singleton, Table}, models::{MaybeToken, Alliance, Permission, CommittedMatchScores, Match, TeamRanking, MatchType}, schedule::generators::MatchGeneratorRPCClient};
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

  // Historical Scores

  #[endpoint]
  async fn get_committed(&self, ctx: &WebsocketContext, _token: &MaybeToken, match_id: String) -> anyhow::Result<CommittedMatchScores> {
    CommittedMatchScores::get(&match_id, &ctx.kv)
  }

  #[endpoint]
  async fn new_committed_record(&self, ctx: &WebsocketContext, token: &MaybeToken, match_id: String) -> anyhow::Result<CommittedMatchScores> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::EditScores])?;
    if CommittedMatchScores::exists(&match_id, &ctx.kv)? {
      anyhow::bail!("There already exists a record for that match!");
    } else {
      let c = CommittedMatchScores { match_id, scores: vec![] };
      c.insert(&ctx.kv)?;
      Ok(c)
    }
  }

  #[endpoint]
  async fn push_committed_score(&self, ctx: &WebsocketContext, token: &MaybeToken, match_id: String, score: MatchScore) -> anyhow::Result<CommittedMatchScores> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::EditScores])?;
    let mut c = CommittedMatchScores::get(&match_id, &ctx.kv)?;
    c.push_and_insert(score, &ctx.kv)?;
    MatchGeneratorRPCClient::update_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;

    Ok(c)
  }

  #[endpoint]
  async fn get_default_scores(&self, _ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<MatchScore> {
    Ok(MatchScore::default())
  }

  #[endpoint]
  async fn derive_score(&self, _ctx: &WebsocketContext, _token: &MaybeToken, score: MatchScore) -> anyhow::Result<MatchScoreSnapshot> {
    Ok(score.into())
  }

  #[endpoint]
  async fn debug_random_fill(&self, ctx: &WebsocketContext, token: &MaybeToken, ty: MatchType) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    for m in Match::all(&ctx.kv)? {
      let match_id = m.id;
      if m.match_type == ty && m.ready {
        match CommittedMatchScores::get(&match_id, &ctx.kv) {
          Ok(mut cms) => {
            if cms.scores.len() == 0 {
              cms.push_and_insert(MatchScore { red: LiveScore::randomise(), blue: LiveScore::randomise() }, &ctx.kv)?;
            }
          },
          Err(_) => {
            let mut c = CommittedMatchScores { match_id: match_id, scores: vec![] };
            c.push_and_insert(MatchScore { red: LiveScore::randomise(), blue: LiveScore::randomise() }, &ctx.kv)?;
          }
        }
      }
    }
    MatchGeneratorRPCClient::update_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }

  // Rankings

  #[publish]
  async fn rankings(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<TeamRanking>> {
    Ok(TeamRanking::sorted(&ctx.kv)?)
  }

  #[endpoint]
  async fn update_rankings(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<Vec<TeamRanking>> {
    TeamRanking::update(&ctx.kv)?;
    Ok(TeamRanking::sorted(&ctx.kv)?)
  }
}