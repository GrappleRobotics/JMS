use jms_core_lib::{models::{Team, MaybeToken, Permission}, db::Table};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait TeamWebsocket {
  #[publish]
  async fn teams(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<Team>> {
    Ok(Team::all(&ctx.kv)?)
  }

  #[endpoint]
  async fn update(&self, ctx: &WebsocketContext, token: &MaybeToken, team: Team) -> anyhow::Result<Team> {
    token.auth(&ctx.kv)?.require_permission(&Permission::Admin)?;
    team.insert(&ctx.kv)?;
    Ok(team)
  }

  #[endpoint]
  async fn delete(&self, ctx: &WebsocketContext, token: &MaybeToken, team_number: usize) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&Permission::Admin)?;
    Team::delete_by(&team_number.to_string(), &ctx.kv)?;
    Ok(())
  }
}