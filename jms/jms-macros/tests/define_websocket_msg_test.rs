use jms_macros::define_websocket_msg;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "state")]
pub enum ArenaState {
  Init,
  Idle { ready: bool }, // Idle state
  Estop,                // Arena is emergency stopped and can only be unlocked by FTA
  EstopReset,           // E-stop resetting...

  // Match Pipeline //
  Prestart { ready: bool },
  MatchArmed,    // Arm the match - ensure field crew is off. Can revert to Prestart.
  MatchPlay,     // Currently running a match - handed off to Match runner
  MatchComplete { ready: bool }, // Match just finished, waiting to commit. Refs can still change scores. Prestart reverts.
  MatchCommit,   // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "signal")]
pub enum ArenaSignal {
  Estop,
  EstopReset,
  Prestart,
  MatchArm,
  MatchPlay,
  MatchCommit,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
pub struct AllianceStation {
  id: u16
}

define_websocket_msg!($ArenaMessage {
  $State{
    send Update(ArenaState),
    recv Signal(ArenaSignal),
    send $AnotherMsg{
      Something(u16),
    },
    send IHaveNoFields
  },
  $Alliances{
    send Update(Vec<AllianceStation>),
    recv SomethingElse
  },
  $SendOnly {
    send Abcd
  }
});

#[test]
pub fn test_schema_gen() {
  // let msg = ArenaMessage2UI::State(ArenaMessageState2UI::AnotherMsg(ArenaMessageStateAnotherMsg2UI::Something(128)));
  let msg = ArenaMessage2UI::State(ArenaMessageState2UI::IHaveNoFields);
  println!("{:?}", msg.ws_path());

  println!("{}", serde_json::to_string_pretty(&msg).unwrap());

  // let schema = schemars::schema_for!(ArenaMessage2UI);
  // println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}