use std::time::Duration;

use jms_base::kv;
use jms_core_lib::{db::{Singleton, Table}, models::{Alliance, CommittedMatchScores, Match, MatchType, MaybeToken, Permission, TeamRanking}, schedule::generators::MatchGeneratorRPCClient, scoring::scores::{LiveScore, MatchScore, MatchScoreSnapshot, ScoreUpdateData, ScoringConfig}};
use uuid::Uuid;

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait ScoringWebsocket {
  // Config
  #[publish]
  async fn config(&self, ctx: &WebsocketContext) -> anyhow::Result<ScoringConfig> {
    let config = ScoringConfig::get(&ctx.kv)?;
    Ok(config)
  }

  #[endpoint]
  async fn update_config(&self, ctx: &WebsocketContext, token: &MaybeToken, config: ScoringConfig) -> anyhow::Result<ScoringConfig> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::EditScores])?;
    ScoringConfig::update(&config, &ctx.kv)?;
    Ok(config)
  }

  #[publish]
  async fn current(&self, ctx: &WebsocketContext) -> anyhow::Result<MatchScoreSnapshot> {
    let config = ScoringConfig::get(&ctx.kv)?;
    Ok(MatchScore::get(&ctx.kv)?.derive(config))
  }

  async fn do_score_update(kv: &kv::KVConnection, update: ScoreUpdateData) -> anyhow::Result<MatchScoreSnapshot> {
    let config = ScoringConfig::get(&kv)?;
    let mut live_score = MatchScore::get(kv)?;
    match update.alliance {
      Alliance::Red => live_score.red.update(update.update),
      Alliance::Blue => live_score.blue.update(update.update)
    }
    live_score.update(kv)?;
    Ok(live_score.derive(config))
  }

  #[endpoint]
  async fn score_update(&self, ctx: &WebsocketContext, token: &MaybeToken, update: ScoreUpdateData) -> anyhow::Result<MatchScoreSnapshot> {
    let hp_permission = match update.alliance {
      Alliance::Blue => Permission::HumanPlayerBlue,
      Alliance::Red => Permission::HumanPlayerRed,
    };
    
    // Check permissions
    match update.update {
      jms_core_lib::scoring::scores::ScoreUpdate::Coop => token.auth(&ctx.kv)?.require_permission(&[hp_permission])?,
      jms_core_lib::scoring::scores::ScoreUpdate::Amplify => token.auth(&ctx.kv)?.require_permission(&[hp_permission])?,
      _ => token.auth(&ctx.kv)?.require_permission(&[Permission::Scoring])?
    };
    
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

  #[endpoint]
  async fn score_full_update(&self, ctx: &WebsocketContext, token: &MaybeToken, score: MatchScore) -> anyhow::Result<MatchScoreSnapshot> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::EditScores])?;

    let my_id = Uuid::new_v4().to_string();
    let config = ScoringConfig::get(&ctx.kv)?;
    
    loop {
      ctx.kv.setnx("__score_update_lock", &my_id)?;
      if ctx.kv.get::<String>("__score_update_lock")? == my_id {
        break;
      }
      tokio::time::sleep(Duration::from_millis(1)).await;
    }

    score.update(&ctx.kv)?;

    ctx.kv.del("__score_update_lock")?;

    Ok(score.derive(config))
  }

  // Historical Scores

  #[publish]
  async fn latest_scores(&self, ctx: &WebsocketContext) -> anyhow::Result<Option<CommittedMatchScores>> {
    let mut scores = CommittedMatchScores::all(&ctx.kv)?.into_iter().filter(|s| s.scores.len() > 0).collect::<Vec<_>>();
    scores.sort_by(|a, b| a.last_update.cmp(&b.last_update));
    Ok(scores.last().cloned())
  }

  #[endpoint]
  async fn get_matches_with_scores(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<Vec<String>> {
    let mut scores = CommittedMatchScores::all(&ctx.kv)?.into_iter().filter(|s| s.scores.len() > 0).collect::<Vec<_>>();
    scores.sort_by(|a, b| b.last_update.cmp(&a.last_update));
    Ok(scores.into_iter().map(|x| x.match_id).collect())
  }

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
      let c = CommittedMatchScores { match_id, scores: vec![], last_update: chrono::Local::now() };
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
  async fn derive_score(&self, ctx: &WebsocketContext, _token: &MaybeToken, score: MatchScore) -> anyhow::Result<MatchScoreSnapshot> {
    let config = ScoringConfig::get(&ctx.kv)?;
    Ok(score.derive(config))
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
            let mut c = CommittedMatchScores { match_id: match_id, scores: vec![], last_update: chrono::Local::now() };
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