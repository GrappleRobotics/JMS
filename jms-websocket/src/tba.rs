use jms_core_lib::{models::{MaybeToken, Permission}, db::Singleton};
use jms_tba_lib::{TBASettings, TBASettingsUpdate, TBARPC, TBARPCClient};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait TBAWebsocket {
  #[endpoint]
  async fn get_settings(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<TBASettings> {
    TBASettings::get(&ctx.kv)
  }

  #[endpoint]
  async fn update_settings(&self, ctx: &WebsocketContext, token: &MaybeToken, update: TBASettingsUpdate) -> anyhow::Result<TBASettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let mut settings = TBASettings::get(&ctx.kv)?;
    update.apply(&mut settings);
    settings.update(&ctx.kv)?;
    Ok(settings)
  }

  #[endpoint]
  async fn update_now(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    TBARPCClient::update_now(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }
}