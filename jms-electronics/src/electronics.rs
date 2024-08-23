use std::{borrow::Cow, net::{IpAddr, Ipv4Addr}, time::Duration};

use binmarshal::AsymmetricCow;
use grapple_frc_msgs::grapple::{jms::{Colour, JMSCardUpdate, JMSElectronicsUpdate, JMSMessage, JMSRole, Pattern}, misc::MiscMessage, GrappleDeviceMessage, TaggedGrappleMessage};
use jms_arena_lib::{AllianceStation, ArenaEntryCondition, ArenaRPCClient, ArenaState, SerialisedLoadedMatch, ARENA_MATCH_KEY, ARENA_STATE_KEY};
use jms_base::{kv, mq::{self, MessageQueueChannel}};
use jms_core_lib::{db::{Singleton, Table}, models::Alliance, scoring::scores::{MatchScore, ScoringConfig}};
use jms_driverstation_lib::DriverStationReport;
use jms_electronics_lib::{EstopMode, FieldElectronicsEndpoint, FieldElectronicsServiceRPC, FieldElectronicsSettings, FieldElectronicsUpdate};
use log::warn;
use pnet::datalink;

use crate::network::JMSElectronicsL2Framed;

pub fn get_jms_admin_interface() -> datalink::NetworkInterface {
  datalink::interfaces().into_iter()
          .filter(|net| !net.is_loopback() && net.is_up() && net.ips.iter().filter(|ip| ip.contains(IpAddr::V4(Ipv4Addr::new(10, 0, 100, 5)))).next().is_some())
          .next()
          .unwrap()
}

pub struct JMSElectronics {
  kv: kv::KVConnection,
  mq: mq::MessageQueueChannel,
}

impl JMSElectronics {
  pub fn new(kv: kv::KVConnection, mq: mq::MessageQueueChannel) -> Self {
    Self { kv, mq }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    // let udp_socket = UdpSocket::bind("0.0.0.0:50002").await?;
    // let mut framed = UdpFramed::new(udp_socket, JMSElectronicsCodec {});
    let mut framed = JMSElectronicsL2Framed::new(get_jms_admin_interface());

    let mut stations = AllianceStation::sorted(&self.kv)?;
    let mut settings = FieldElectronicsSettings::get(&self.kv)?;
    let mut endpoints = FieldElectronicsEndpoint::all(&self.kv)?;
    let mut score = MatchScore::get(&self.kv)?;
    let mut entry_condition = ArenaEntryCondition::get(&self.kv)?;
    let mut current_match: Option<SerialisedLoadedMatch> = self.kv.json_get(ARENA_MATCH_KEY, "$").ok();

    // let mut arena_is_estopped = false;
    let mut arena_state = self.kv.json_get::<ArenaState>(ARENA_STATE_KEY, "$")?;

    let mut cache_interval = tokio::time::interval(Duration::from_millis(100));
    let mut lighting_update = tokio::time::interval(Duration::from_millis(250));

    let mut tick_n: u8 = 0;

    loop {
      tokio::select! {
        _ = cache_interval.tick() => {
          stations = AllianceStation::sorted(&self.kv)?;
          settings = FieldElectronicsSettings::get(&self.kv)?;
          endpoints = FieldElectronicsEndpoint::all(&self.kv)?;
          score = MatchScore::get(&self.kv)?;
          arena_state = self.kv.json_get::<ArenaState>(ARENA_STATE_KEY, "$")?;
          entry_condition = ArenaEntryCondition::get(&self.kv)?;
          current_match = self.kv.json_get(ARENA_MATCH_KEY, "$").ok();
        },
        _ = lighting_update.tick() => {
          tick_n = tick_n.wrapping_add(1);
          let config = ScoringConfig::get(&self.kv)?;

          for station in &stations {
            let (team_score, other_score, primary_colour, secondary_colour, target_role) = match station.id.alliance {
              Alliance::Blue => ( &score.blue, &score.red, Colour::new(0, 0, 255), Colour::new(0, 0, 5), JMSRole::Blue(station.id.station as u8) ),
              Alliance::Red => ( &score.red, &score.blue, Colour::new(255, 0, 0), Colour::new(5, 0, 0), JMSRole::Red(station.id.station as u8) ),
            };

            let score_derived = team_score.derive(&other_score, config);

            let top_bar = match (team_score.coop, other_score.coop, arena_state) {
              (_, _, ArenaState::Estop) => Pattern::Blank,
              (true, true, _) => Pattern::Solid(Colour::new(255, 120, 0)),
              (true, false, _) => Pattern::FillLeft(Colour::new(255, 255, 0), Colour::new(5, 5, 0), 128),
              _ => Pattern::Blank
            };

            let report = station.team.and_then(|team| DriverStationReport::get(&(team as u16), &self.kv).ok());
            
            let (bottom_bar, back_background, back_text, back_colour) = match (station.astop, station.estop, report, arena_state) {
              (_, _, _, ArenaState::Estop) => {
                if tick_n % 8 < 4 {
                  (Pattern::Blank, Pattern::Blank, "ARENA".to_owned(), Colour::new(255, 0, 0))
                } else {
                  (Pattern::Blank, Pattern::Blank, "FAULT".to_owned(), Colour::new(255, 0, 0))
                }
              },
              (_, true, _, _) => (Pattern::DiagonalStripes(Colour::new(255, 0, 0), Colour::new(5, 0, 0)), Pattern::Blank, "ESTOP".to_owned(), Colour::new(255, 0, 0)),
              (true, _, _, _) => (Pattern::DiagonalStripes(Colour::new(255, 80, 0), Colour::new(5, 5, 0)), Pattern::Blank, "ASTOP".to_owned(), Colour::new(255, 80, 0)),
              (_, _, Some(report), _) if report.diagnosis() != None => {
                (Pattern::Solid(Colour::new(255, 80, 0)), Pattern::Blank, report.diagnosis().unwrap().to_owned(), Colour::new(255, 0, 0))
              },
              (_, _, _, ArenaState::MatchPlay) => {
                if let Some(secs) = score_derived.notes.amplified_remaining.as_ref().map(|x| x.0.num_seconds()) {
                  (
                    Pattern::Blank,
                    Pattern::FillLeft(primary_colour.clone(), secondary_colour.clone(), (255 / 10) * secs as u8),
                    "".to_owned(),
                    Colour::new(0, 0, 0)
                  )
                } else {
                  (
                    Pattern::Blank,
                    Pattern::Blank,
                    format!("{}/{}", score_derived.notes.total_count, score_derived.melody_threshold),
                    if score_derived.melody_rp { Colour::new(0, 255, 0) } else { Colour::new(255, 40, 0) }
                  )
                }
              },
              _ => (Pattern::Blank, Pattern::Blank, station.team.map(|team| format!("{}", team)).unwrap_or("----".to_owned()), Colour::new(0, 255, 0))
            };

            let (background, text_colour) = match (arena_state, score_derived.notes.amplified_remaining, entry_condition.clone()) {
              (ArenaState::Estop, _, _) => {
                if tick_n % 4 < 2 {
                  (
                    Pattern::DiagonalStripes(Colour::new(120, 0, 0), Colour::new(5, 0, 0)),
                    Colour::new(0, 0, 0)
                  )
                } else {
                  (
                    Pattern::DiagonalStripes(Colour::new(5, 0, 0), Colour::new(120, 0, 0)),
                    Colour::new(0, 0, 0)
                  )
                }
              },
              (_, _, ArenaEntryCondition::Safe) => {
                (Pattern::Blank, Colour::new(0, 255, 0))
              },
              (_, _, ArenaEntryCondition::Reset) => {
                (Pattern::Blank, Colour::new(255, 0, 255))
              },
              (_, Some(x), _) if x.0.num_seconds() <= 10 => {
                (
                  Pattern::FillLeft(primary_colour.clone(), secondary_colour, (255 / 10) * x.0.num_seconds() as u8),
                  Colour::new(0, 0, 0)
                )
              },
              _ => (Pattern::Blank, primary_colour.clone())
            };

            let front_text = match (station.team, arena_state, entry_condition.clone()) {
              (_, ArenaState::Estop, _) => {
                if tick_n % 8 < 4 {
                  "ARENA".to_owned()
                } else {
                  "FAULT".to_owned()
                }
              },
              (_, _, ArenaEntryCondition::Safe) => {
                "SAFE".to_owned()
              },
              (_, _, ArenaEntryCondition::Reset) => {
                "RESET".to_owned()
              },
              (Some(team), _, ArenaEntryCondition::Unsafe) => format!("{}", team),
              (None, _, ArenaEntryCondition::Unsafe) => "----".to_owned(),
            };

            for ep in &endpoints {
              if ep.status.role == target_role {
                framed.send_framed(
                  ep.mac.parse()?,
                  TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::Update(JMSElectronicsUpdate {
                    card: 1,
                    update: JMSCardUpdate::Lighting {
                      text_back: AsymmetricCow(Cow::Borrowed(&back_text)),
                      text_back_colour: back_colour.clone(),
                      back_background: back_background.clone(),
                      text: AsymmetricCow(Cow::Borrowed(&front_text)),
                      text_colour: text_colour.clone(),
                      bottom_bar: bottom_bar.clone(),
                      top_bar: top_bar.clone(),
                      background: background.clone()
                    }
                  }))))
                ).await.map_err(|e| e.to_string()).ok();
              }
            }
          }

          for ep in &endpoints {
            if [JMSRole::TimerBlue, JMSRole::TimerRed, JMSRole::ScoringTable].contains(&ep.status.role) {
              let mut front_text = "---".to_owned();
              let mut portion = 0;

              if let Some(current) = current_match.as_ref() {
                front_text = format!("{}", current.remaining.0.num_seconds());
                if current.remaining_max.0.num_seconds() > 0 {
                  portion = ((current.remaining.0.num_seconds() * 255) / (current.remaining_max.0.num_seconds())) as u8;
                }
              }

              framed.send_framed(
                ep.mac.parse()?,
                TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::Update(JMSElectronicsUpdate {
                  card: 1,
                  update: JMSCardUpdate::Lighting {
                    text_back: AsymmetricCow(Cow::Borrowed("")),
                    text_back_colour: Colour::new(0, 0, 0).clone(),
                    back_background: Pattern::Blank,
                    text: AsymmetricCow(Cow::Borrowed(&front_text)),
                    text_colour: Colour::new(255, 255, 255),
                    bottom_bar: Pattern::FillLeft(Colour::new(255, 255, 255), Colour::new(0, 0, 0), portion),
                    top_bar: Pattern::Blank,
                    background: Pattern::Blank
                  }
                }))))
              ).await.map_err(|e| e.to_string()).ok();
            }
          }
        },
        msg = framed.next_framed() => match msg {
          Some((endpoint, data)) => match data.msg {
            GrappleDeviceMessage::Misc(MiscMessage::JMS(jms)) => match jms {
              JMSMessage::Status(status) => {
                let ep = FieldElectronicsEndpoint {
                  mac: endpoint.to_string(),
                  status
                };
                ep.insert(&self.kv).ok();
                ep.expire(1, &self.kv).ok();

                let invert = settings.estop_mode == EstopMode::NormallyClosed;

                match ep.status.cards[0] {
                  grapple_frc_msgs::grapple::jms::JMSCardStatus::IO(io) => {
                    match ep.status.role {
                      JMSRole::ScoringTable => {
                        let estop_state = invert ^ io[0];
                        if estop_state && arena_state != ArenaState::Estop {
                          match ArenaRPCClient::signal(&self.mq, jms_arena_lib::ArenaSignal::Estop, "Field Electronics (Scoring Table)".to_string()).await? {
                            Ok(()) => (),
                            Err(e) => warn!("Field Electronics - Signal Error: {}", e)
                          }
                        }
                      },
                      JMSRole::Red(stn) => {
                        if let Some(stn) = stations.iter_mut().find(|x| x.id.alliance == Alliance::Red && x.id.station == stn as usize) {
                          let estop_state = invert ^ io[0];
                          if estop_state != stn.physical_estop {
                            stn.set_physical_estop(estop_state, &self.kv)?;
                          }
                        }
                      },
                      JMSRole::Blue(stn) => {
                        if let Some(stn) = stations.iter_mut().find(|x| x.id.alliance == Alliance::Blue && x.id.station == stn as usize) {
                          let estop_state = invert ^ io[0];
                          if estop_state != stn.physical_estop {
                            stn.set_physical_estop(estop_state, &self.kv)?;
                          }
                        }
                      },
                      _ => ()
                    }
                  },
                  grapple_frc_msgs::grapple::jms::JMSCardStatus::Lighting => {},
                }
              },
              _ => ()
            },
            _ => ()
          },
          // Some(Err(e)) => error!("Error: {}", e),
          None => (),
        }
      }
    }
  }
}

/* TODO: We can move this into JMSElectronics and use tokio::select! on the rpc.next() option to keep
  mutability requirements.  */
pub struct JMSElectronicsService {
  pub mq: MessageQueueChannel,
  pub kv: kv::KVConnection,
  pub framed: JMSElectronicsL2Framed
}

impl JMSElectronicsService {
  pub async fn new(mq: MessageQueueChannel, kv: kv::KVConnection) -> Result<Self, anyhow::Error> {
    // let udp_socket = UdpSocket::bind("0.0.0.0:50003").await?;
    // let framed = UdpFramed::new(udp_socket, JMSElectronicsCodec {});
    let framed = JMSElectronicsL2Framed::new(get_jms_admin_interface());

    Ok(Self { mq, kv, framed })
  }

  pub async fn run(mut self) -> anyhow::Result<()> {
    let mut rpc = self.rpc_handle().await?;
    loop {
      self.rpc_process(rpc.next().await).await?;
    }
  }
}

#[async_trait::async_trait]
impl FieldElectronicsServiceRPC for JMSElectronicsService {
  fn mq(&self) -> &MessageQueueChannel { &self.mq }

  async fn update(&mut self, update: FieldElectronicsUpdate) -> Result<(), String> {
    match update {
      FieldElectronicsUpdate::SetRole { mac, role } => {
        if let Ok(mac) = mac.parse() {
          self.framed.send_framed(
            mac,
            TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::SetRole(role)))),
          ).await.map_err(|e| e.to_string())
        } else {
          Err(format!("Invalid MAC: {}", mac))
        }
      },
      FieldElectronicsUpdate::Blink { mac } => {
        if let Ok(mac) = mac.parse() {
          self.framed.send_framed(
            mac,
            TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::Blink))),
          ).await.map_err(|e| e.to_string())
        } else {
          Err(format!("Invalid MAC: {}", mac))
        }
      }
    }
  }

  async fn reset_estops(&mut self) -> Result<(), String> {
    for mut stn in AllianceStation::all(&self.kv).map_err(|e| e.to_string())? {
      stn.set_physical_estop(false, &self.kv).map_err(|e| e.to_string())?;
    }
    Ok(())
  }
}