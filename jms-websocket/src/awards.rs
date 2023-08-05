use jms_core_lib::{models::{Award, MaybeToken, Permission}, db::Table};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait AwardsWebsocket {

  #[publish]
  async fn awards(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<Award>> {
    Award::all(&ctx.kv)
  }

  #[endpoint]
  async fn set_award(&self, ctx: &WebsocketContext, token: &MaybeToken, award: Award) -> anyhow::Result<Award> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAwards])?;
    award.insert(&ctx.kv)?;
    Ok(award)
  }

  #[endpoint]
  async fn delete_award(&self, ctx: &WebsocketContext, token: &MaybeToken, award_id: String) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAwards])?;
    Award::delete_by(&award_id, &ctx.kv)?;
    Ok(())
  }
}