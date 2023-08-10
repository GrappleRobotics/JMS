use jms_base::kv;

use crate::db::Singleton;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "scene", content = "params")]
pub enum AudienceDisplayScene {
  Blank,
  MatchPreview,
  MatchPlay,
  MatchResults(/* Match ID */ String),
  AllianceSelection,
  PlayoffBracket,
  Award(/* Award ID */ String),
  CustomMessage(String),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum AudienceDisplaySound {
  AutoStart,
  TeleopStart,
  Endgame,
  Estop,
  MatchStop,
}


#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AudienceDisplay {
  pub scene: AudienceDisplayScene,
  pub queued_sound: Option<AudienceDisplaySound>
}

impl Default for AudienceDisplay {
  fn default() -> Self {
    Self { scene: AudienceDisplayScene::Blank, queued_sound: None }
  }
}

impl Singleton for AudienceDisplay {
  const KEY: &'static str = "audience";
}

impl AudienceDisplay {
  pub fn set_scene(scene: AudienceDisplayScene, kv: &kv::KVConnection) -> anyhow::Result<()> {
    kv.json_set(Self::KEY, "$.scene", &scene)
  }

  pub fn play_sound(sound: AudienceDisplaySound, kv: &kv::KVConnection) -> anyhow::Result<()> {
    kv.json_set(Self::KEY, "$.queued_sound", &Some(sound))
  }

  pub fn take_sound(&mut self, kv: &kv::KVConnection) -> anyhow::Result<Option<AudienceDisplaySound>> {
    let s = self.queued_sound.take();
    kv.json_set(Self::KEY, "$.queued_sound", &Option::<AudienceDisplaySound>::None)?;
    Ok(s)
  }
}