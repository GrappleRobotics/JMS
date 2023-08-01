use jms_core_lib::{models::{Team, MaybeToken, Permission, TeamUpdate}, db::Table};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait TeamWebsocket {
  #[publish]
  async fn teams(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<Team>> {
    Ok(Team::all(&ctx.kv)?)
  }

  #[endpoint]
  async fn new_team(&self, ctx: &WebsocketContext, token: &MaybeToken, team_number: usize, display_number: String, name: Option<String>, affiliation: Option<String>, location: Option<String>) -> anyhow::Result<Team> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageTeams])?;
    let team = Team {
      number: team_number,
      display_number, affiliation, location, name,
      notes: None, wpakey: None, schedule: true
    };
    team.insert(&ctx.kv)?;
    Ok(team)
  }

  #[endpoint]
  async fn update(&self, ctx: &WebsocketContext, token: &MaybeToken, team_number: usize, updates: Vec<TeamUpdate>) -> anyhow::Result<Team> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageTeams])?;
    let mut team = Team::get(&team_number, &ctx.kv)?;
    for update in updates {
      update.apply(&mut team);
    }
    team.insert(&ctx.kv)?;
    Ok(team)
  }

  #[endpoint]
  async fn delete(&self, ctx: &WebsocketContext, token: &MaybeToken, team_number: usize) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageTeams])?;
    Team::delete_by(&team_number, &ctx.kv)?;
    Ok(())
  }
}