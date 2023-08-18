use jms_backup_lib::{BackupSettings, BackupSettingsUpdate, JMSBackupRPCClient};
use jms_core_lib::{models::{MaybeToken, Permission}, db::Singleton};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait BackupWebsocket {

  #[endpoint]
  async fn settings(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<BackupSettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    BackupSettings::get(&ctx.kv)
  }

  #[endpoint]
  async fn update_settings(&self, ctx: &WebsocketContext, token: &MaybeToken, update: BackupSettingsUpdate) -> anyhow::Result<BackupSettings> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let mut settings = BackupSettings::get(&ctx.kv)?;
    update.apply(&mut settings);
    settings.update(&ctx.kv)?;
    Ok(settings)
  }

  #[endpoint]
  async fn backup_now(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    JMSBackupRPCClient::backup_now(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }

  #[endpoint]
  async fn backup_to(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<String> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    let data = JMSBackupRPCClient::backup_to(&ctx.mq).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(std::str::from_utf8(&data[..])?.to_owned())
  }

  #[endpoint]
  async fn restore(&self, ctx: &WebsocketContext, token: &MaybeToken, data: String) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::FTA])?;
    JMSBackupRPCClient::restore(&ctx.mq, data.into_bytes()).await?.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
  }
}