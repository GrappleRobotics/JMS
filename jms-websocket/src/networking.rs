use jms_core_lib::{models::{MaybeToken, Permission}, db::Singleton};
use jms_networking_lib::{NetworkingSettings, NetworkingSettingsUpdate, JMSNetworkingRPCClient};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait NetworkingWebsocket {
  
  #[endpoint]
  async fn settings(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<NetworkingSettings> {
    NetworkingSettings::get(&ctx.kv)
  }

  #[endpoint]
  async fn update_settings(&self, ctx: &WebsocketContext, token: &MaybeToken, update: NetworkingSettingsUpdate) -> anyhow::Result<NetworkingSettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let mut settings = NetworkingSettings::get(&ctx.kv)?;
    update.apply(&mut settings);
    settings.update(&ctx.kv)?;
    Ok(settings)
  }

  #[endpoint]
  async fn reload_admin(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    JMSNetworkingRPCClient::configure_admin(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))
  }
}