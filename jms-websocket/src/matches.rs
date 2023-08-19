use jms_core_lib::{models::{Match, MaybeToken, Permission, PlayoffMode, CommittedMatchScores, TeamRanking}, db::{Table, Singleton}, schedule::generators::{QualsMatchGeneratorParams, MatchGeneratorRPCClient, MATCH_GENERATOR_JOB_KEY}};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait MatchesWebsocket {
  // Matches

  #[publish]
  async fn matches(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<Match>> {
    Match::sorted(&ctx.kv)
  }

  #[publish]
  async fn next(&self, ctx: &WebsocketContext) -> anyhow::Result<Option<Match>> {
    Ok(Match::sorted(&ctx.kv)?.into_iter().find(|m| !m.played))
  }

  #[publish]
  async fn generator_busy(&self, ctx: &WebsocketContext) -> anyhow::Result<bool> {
    ctx.kv.exists(MATCH_GENERATOR_JOB_KEY)
  }

  #[endpoint]
  async fn delete(&self, ctx: &WebsocketContext, token: &MaybeToken, match_id: String) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageSchedule])?;
    let m = Match::get(&match_id, &ctx.kv)?;
    if m.played {
      anyhow::bail!("Can't delete a match that's already been played!")
    } else {
      m.delete(&ctx.kv)?;
    }
    Ok(())
  }

  #[endpoint]
  async fn debug_delete_all(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    Match::clear(&ctx.kv)?;
    CommittedMatchScores::clear(&ctx.kv)?;

    TeamRanking::update(&ctx.kv)?;
    Ok(())
  }

  // Quals

  #[endpoint]
  async fn gen_quals(&self, ctx: &WebsocketContext, token: &MaybeToken, params: QualsMatchGeneratorParams) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageSchedule])?;
    MatchGeneratorRPCClient::start_qual_gen(&ctx.mq, params).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }

  // Playoffs

  #[endpoint]
  async fn get_playoff_mode(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<PlayoffMode> {
    Ok(PlayoffMode::get(&ctx.kv)?)
  }

  #[endpoint]
  async fn set_playoff_mode(&self, ctx: &WebsocketContext, token: &MaybeToken, mode: PlayoffMode) -> anyhow::Result<PlayoffMode> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManagePlayoffs])?;
    MatchGeneratorRPCClient::reset_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    mode.update(&ctx.kv)?;
    MatchGeneratorRPCClient::update_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(mode)
  }

  #[endpoint]
  async fn reset_playoffs(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManagePlayoffs])?;
    MatchGeneratorRPCClient::reset_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }

  #[endpoint]
  async fn update_playoffs(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManagePlayoffs])?;
    MatchGeneratorRPCClient::update_playoffs(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }
}
