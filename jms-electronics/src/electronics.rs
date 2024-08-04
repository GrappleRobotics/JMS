use std::{borrow::Cow, net::{IpAddr, SocketAddr}, str::FromStr, time::Duration};

use binmarshal::{AsymmetricCow, BitView, BitWriter, Demarshal, VecBitWriter};
use bounded_static::ToBoundedStatic;
use bytes::Buf;
use futures::{SinkExt, StreamExt};
use grapple_frc_msgs::{grapple::{jms::{Colour, JMSCardUpdate, JMSElectronicsUpdate, JMSMessage, JMSRole, Pattern}, misc::MiscMessage, write_direct, GrappleDeviceMessage, GrappleMessageId, MaybeFragment, TaggedGrappleMessage}, ManufacturerMessage, MessageId};
use jms_arena_lib::{AllianceStation, ArenaRPCClient, ArenaState, ARENA_STATE_KEY};
use jms_base::{kv, mq::{self, MessageQueue, MessageQueueChannel}};
use jms_core_lib::{db::{Singleton, Table}, models::Alliance, scoring::scores::MatchScore};
use jms_driverstation_lib::DriverStationReport;
use jms_electronics_lib::{EstopMode, FieldElectronicsEndpoint, FieldElectronicsServiceRPC, FieldElectronicsSettings, FieldElectronicsUpdate};
use log::{error, warn};
use tokio::net::UdpSocket;
use tokio_util::{codec::{Decoder, Encoder}, udp::UdpFramed};

pub struct JMSElectronics {
  kv: kv::KVConnection,
  mq: mq::MessageQueueChannel,
}

impl JMSElectronics {
  pub fn new(kv: kv::KVConnection, mq: mq::MessageQueueChannel) -> Self {
    Self { kv, mq }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    let udp_socket = UdpSocket::bind("0.0.0.0:50002").await?;
    let mut framed = UdpFramed::new(udp_socket, JMSElectronicsCodec {});

    let mut stations = AllianceStation::sorted(&self.kv)?;
    let mut settings = FieldElectronicsSettings::get(&self.kv)?;
    let mut endpoints = FieldElectronicsEndpoint::all(&self.kv)?;
    let mut score = MatchScore::get(&self.kv)?;

    let mut arena_is_estopped = false;

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
          arena_is_estopped = self.kv.json_get::<ArenaState>(ARENA_STATE_KEY, "$")? == ArenaState::Estop;
        },
        _ = lighting_update.tick() => {
          tick_n = tick_n.wrapping_add(1);
          for station in &stations {
            let (team_score, other_score, primary_colour, secondary_colour, target_role) = match station.id.alliance {
              Alliance::Blue => ( &score.blue, &score.red, Colour::new(0, 0, 255), Colour::new(0, 0, 5), JMSRole::Blue(station.id.station as u8) ),
              Alliance::Red => ( &score.red, &score.blue, Colour::new(255, 0, 0), Colour::new(5, 0, 0), JMSRole::Red(station.id.station as u8) ),
            };

            let score_derived = team_score.derive(&other_score);

            let top_bar = match (team_score.coop, other_score.coop, arena_is_estopped) {
              (true, true, false) => Pattern::Solid(Colour::new(255, 120, 0)),
              (true, false, false) => Pattern::FillLeft(Colour::new(255, 255, 0), Colour::new(5, 5, 0), 128),
              _ => Pattern::Blank
            };

            let report = station.team.and_then(|team| DriverStationReport::get(&(team as u16), &self.kv).ok());
            let (bottom_bar, back_text, back_colour) = match (station.astop, station.estop, report, arena_is_estopped) {
              (_, _, _, true) => {
                if tick_n % 8 < 4 {
                  (Pattern::Blank, "ARENA".to_owned(), Colour::new(255, 0, 0))
                } else {
                  (Pattern::Blank, "FAULT".to_owned(), Colour::new(255, 0, 0))
                }
              },
              (_, true, _, _) => (Pattern::DiagonalStripes(Colour::new(255, 0, 0), Colour::new(5, 0, 0)), "ESTOP".to_owned(), Colour::new(255, 0, 0)),
              (true, _, _, _) => (Pattern::DiagonalStripes(Colour::new(255, 80, 0), Colour::new(5, 5, 0)), "ASTOP".to_owned(), Colour::new(255, 80, 0)),
              (_, _, Some(report), _) if report.diagnosis() != None => {
                (Pattern::Solid(Colour::new(255, 80, 0)), report.diagnosis().unwrap().to_owned(), Colour::new(255, 0, 0))
              },
              _ => (Pattern::Blank, station.team.map(|team| format!("{}", team)).unwrap_or("----".to_owned()), Colour::new(0, 255, 0))
            };

            let (background, text_colour) = match (arena_is_estopped, score_derived.notes.amplified_remaining) {
              (true, _) => {
                if tick_n % 4 < 2 {
                  (
                    Pattern::DiagonalStripes(Colour::new(255, 0, 0), Colour::new(5, 0, 0)),
                    Colour::new(255, 255, 255)
                  )
                } else {
                  (
                    Pattern::DiagonalStripes(Colour::new(5, 0, 0), Colour::new(255, 0, 0)),
                    Colour::new(255, 255, 255)
                  )
                }
              }
              (false, Some(x)) if x.0.num_seconds() <= 10 => {
                (
                  Pattern::FillLeft(primary_colour.clone(), secondary_colour, (255 / 10) * x.0.num_seconds() as u8),
                  Colour::new(0, 0, 0)
                )
              },
              _ => (Pattern::Blank, primary_colour.clone())
            };

            let front_text = match (station.team, arena_is_estopped) {
              (_, true) => {
                if tick_n % 8 < 4 {
                  "ARENA".to_owned()
                } else {
                  "FAULT".to_owned()
                }
              },
              (Some(team), false) => format!("{}", team),
              (None, false) => "----".to_owned(),
            };

            for ep in &endpoints {
              if ep.status.role == target_role {
                framed.send((
                  TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::Update(JMSElectronicsUpdate {
                    card: 1,
                    update: JMSCardUpdate::Lighting {
                      text_back: AsymmetricCow(Cow::Borrowed(&back_text)),
                      text_back_colour: back_colour.clone(),
                      text: AsymmetricCow(Cow::Borrowed(&front_text)),
                      text_colour: text_colour.clone(),
                      bottom_bar: bottom_bar.clone(),
                      top_bar: top_bar.clone(),
                      background: background.clone()
                    }
                  })))),
                  SocketAddr::new(ep.ip.parse().unwrap(), 50002)
                )).await.map_err(|e| e.to_string()).ok();
              }
            }
          }
        },
        msg = framed.next() => match msg {
          Some(Ok((data, endpoint))) => match data.msg {
            GrappleDeviceMessage::Misc(MiscMessage::JMS(jms)) => match jms {
              JMSMessage::Status(status) => {
                let ep = FieldElectronicsEndpoint {
                  ip: endpoint.ip().to_string(),
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
                        if estop_state && !arena_is_estopped {
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
          Some(Err(e)) => error!("Error: {}", e),
          None => (),
        }
      }
    }
  }
}

pub struct JMSElectronicsCodec { }

impl Decoder for JMSElectronicsCodec {
  type Item = TaggedGrappleMessage<'static>;
  type Error = anyhow::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.len() < 4 { return Ok(None) }

    let mut view = BitView::new(&src[..]);

    let result = match view.take::<4>(4, 0) {
      Ok(arr) => {
        let id: MessageId = u32::from_le_bytes(*arr.0).into();
        match ManufacturerMessage::read(&mut view, id.clone()) {
          Ok(ManufacturerMessage::Grapple(MaybeFragment::Message(msg))) => {
            Ok(Some(TaggedGrappleMessage::new(id.device_id, msg.to_static())))
          },
          _ => Ok(None),
        }
      },
      Err(_) => Err(anyhow::anyhow!("Demarshal Error Error"))
    };

    src.advance(src.len());

    result
  }
}

impl<'a> Encoder<TaggedGrappleMessage<'a>> for JMSElectronicsCodec {
  type Error = anyhow::Error;

  fn encode(&mut self, item: TaggedGrappleMessage<'a>, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    let mut writer = VecBitWriter::new();
    write_direct(&mut writer, item).map_err(|_| anyhow::anyhow!("Writer Error!"))?;
    dst.extend_from_slice(writer.slice());
    Ok(())
  }
}

/* TODO: We can move this into JMSElectronics and use tokio::select! on the rpc.next() option to keep
  mutability requirements.  */
pub struct JMSElectronicsService {
  pub mq: MessageQueueChannel,
  pub kv: kv::KVConnection,
  pub framed: UdpFramed<JMSElectronicsCodec, UdpSocket>
}

impl JMSElectronicsService {
  pub async fn new(mq: MessageQueueChannel, kv: kv::KVConnection) -> Result<Self, anyhow::Error> {
    let udp_socket = UdpSocket::bind("0.0.0.0:50003").await?;
    let framed = UdpFramed::new(udp_socket, JMSElectronicsCodec {});

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
      FieldElectronicsUpdate::SetRole { ip, role } => {
        if let Ok(ip) = ip.parse() {
          self.framed.send((
            TaggedGrappleMessage::new(0x00, GrappleDeviceMessage::Misc(MiscMessage::JMS(JMSMessage::SetRole(role)))),
            SocketAddr::new(ip, 50002)
          )).await.map_err(|e| e.to_string())
        } else {
          Err(format!("Invalid IP: {}", ip))
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