use jms_core_lib::{db::{Singleton, Table}, models::{AudienceDisplay, AudienceDisplayScene, AudienceDisplaySound, CommittedMatchScores, MaybeToken, Permission}};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait AudienceWebsocket {

  #[publish]
  async fn current(&self, ctx: &WebsocketContext) -> anyhow::Result<AudienceDisplay> {
    let ad = AudienceDisplay::get(&ctx.kv)?;
    // TODO: This doesn't work when using multiple websocket instances :( Need to keep track of sound
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
  async fn set_latest_scores(&self, ctx: &WebsocketContext, token: &MaybeToken) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAudience])?;

    let mut scores = CommittedMatchScores::all(&ctx.kv)?.into_iter().filter(|s| s.scores.len() > 0).collect::<Vec<_>>();
    scores.sort_by(|a, b| a.last_update.cmp(&b.last_update));
    
    if let Some(last_match) = scores.last() {
      AudienceDisplay::set_scene(AudienceDisplayScene::MatchResults(last_match.match_id.clone()), &ctx.kv)?;
    }
    Ok(())
  }

  #[endpoint]
  async fn play_sound(&self, ctx: &WebsocketContext, token: &MaybeToken, sound: AudienceDisplaySound) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageAudience])?;
    AudienceDisplay::play_sound(sound, &ctx.kv)?;
    Ok(())
  }
}
