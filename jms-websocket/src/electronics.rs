use jms_core_lib::{db::{Singleton, Table}, models::{MaybeToken, Permission}};
use jms_electronics_lib::{FieldElectronicsEndpoint, FieldElectronicsServiceRPCClient, FieldElectronicsSettings, FieldElectronicsSettingsUpdate, FieldElectronicsUpdate};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait ElectronicsWebsocket {
  #[publish]
  async fn endpoints(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<FieldElectronicsEndpoint>> {
    FieldElectronicsEndpoint::all(&ctx.kv)
  }

  #[endpoint]
  async fn update(&self, ctx: &WebsocketContext, token: &MaybeToken, update: FieldElectronicsUpdate) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageElectronics])?;
    FieldElectronicsServiceRPCClient::update(&ctx.mq, update).await?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn reset_estops(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageElectronics])?;
    FieldElectronicsServiceRPCClient::reset_estops(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))
  }

  #[endpoint]
  async fn settings(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<FieldElectronicsSettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageElectronics])?;
    FieldElectronicsSettings::get(&ctx.kv)
  }

  #[endpoint]
  async fn update_settings(&self, ctx: &WebsocketContext, token: &MaybeToken, update: FieldElectronicsSettingsUpdate) -> anyhow::Result<FieldElectronicsSettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageElectronics])?;
    let mut settings = FieldElectronicsSettings::get(&ctx.kv)?;
    update.apply(&mut settings);
    settings.update(&ctx.kv)?;
    Ok(settings)
  }
}