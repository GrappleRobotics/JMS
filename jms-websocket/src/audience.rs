use jms_core_lib::{models::{AudienceDisplay, MaybeToken, Permission, AudienceDisplaySound, AudienceDisplayScene}, db::Singleton};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait AudienceWebsocket {

  #[publish]
  async fn current(&self, ctx: &WebsocketContext) -> anyhow::Result<AudienceDisplay> {
    let ad = AudienceDisplay::get(&ctx.kv)?;
    let mut ad2 = ad.clone();
    ad2.take_sound(&ctx.kv)?;
    Ok(ad)
  }

  #[endpoint]
  async fn set(&self, ctx: &WebsocketContext, token: &MaybeToken, scene: AudienceDisplayScene) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAudience])?;
    AudienceDisplay::set_scene(scene, &ctx.kv)?;
    Ok(())
  }

  #[endpoint]
  async fn play_sound(&self, ctx: &WebsocketContext, token: &MaybeToken, sound: AudienceDisplaySound) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAudience])?;
    AudienceDisplay::play_sound(sound, &ctx.kv)?;
    Ok(())
  }
}
