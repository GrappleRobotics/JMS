use jms_core_lib::{models::{PlayoffAlliance, MaybeToken, PlayoffMode, Permission}, db::{Singleton, Table}};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait AlliancesWebsocket {
  #[publish]
  async fn alliances(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<PlayoffAlliance>> {
    PlayoffAlliance::sorted(&ctx.kv)
  }

  #[endpoint]
  async fn create(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<Vec<PlayoffAlliance>> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAlliances])?;
    let n = match PlayoffMode::get(&ctx.kv)? {
      PlayoffMode::Bracket { n_alliances } => n_alliances,
      PlayoffMode::DoubleBracket { n_alliances, .. } => n_alliances,
      PlayoffMode::RoundRobin { n_alliances } => n_alliances
    };
    PlayoffAlliance::create_all(n, &ctx.kv)?;
    PlayoffAlliance::sorted(&ctx.kv)
  }

  #[endpoint]
  async fn delete_all(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAlliances])?;
    PlayoffAlliance::clear(&ctx.kv)?;
    Ok(())
  }

  #[endpoint]
  async fn promote(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<Vec<PlayoffAlliance>> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAlliances])?;
    PlayoffAlliance::promote(&ctx.kv)?;
    PlayoffAlliance::sorted(&ctx.kv)
  }

  #[endpoint]
  async fn set_teams(&self, ctx: &WebsocketContext, token: &MaybeToken, number: usize, teams: Vec<usize>) -> anyhow::Result<PlayoffAlliance> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAlliances])?;
    let mut alliance = PlayoffAlliance::get(&number, &ctx.kv)?;
    alliance.teams = teams;
    alliance.insert(&ctx.kv)?;
    Ok(alliance)
  }
}