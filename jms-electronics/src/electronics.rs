use std::{net::{IpAddr, SocketAddr}, str::FromStr, time::Duration};

use binmarshal::{BitView, BitWriter, Demarshal, VecBitWriter};
use bounded_static::ToBoundedStatic;
use bytes::Buf;
use futures::{SinkExt, StreamExt};
use grapple_frc_msgs::{grapple::{jms::{JMSMessage, JMSRole}, misc::MiscMessage, write_direct, GrappleDeviceMessage, GrappleMessageId, MaybeFragment, TaggedGrappleMessage}, ManufacturerMessage, MessageId};
use jms_arena_lib::{AllianceStation, ArenaRPCClient, ArenaState, ARENA_STATE_KEY};
use jms_base::{kv, mq::{self, MessageQueue, MessageQueueChannel}};
use jms_core_lib::{db::{Singleton, Table}, models::Alliance};
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

    let mut arena_is_estopped = false;

    let mut cache_interval = tokio::time::interval(Duration::from_millis(100));

    loop {
      tokio::select! {
        _ = cache_interval.tick() => {
          stations = AllianceStation::all(&self.kv)?;
          settings = FieldElectronicsSettings::get(&self.kv)?;
          arena_is_estopped = self.kv.json_get::<ArenaState>(ARENA_STATE_KEY, "$")? == ArenaState::Estop;
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

                match ep.status.role {
                  JMSRole::ScoringTable => {
                    let estop_state = invert ^ ep.status.cards[0].io_status[0];
                    if estop_state && !arena_is_estopped {
                      match ArenaRPCClient::signal(&self.mq, jms_arena_lib::ArenaSignal::Estop, "Field Electronics (Scoring Table)".to_string()).await? {
                        Ok(()) => (),
                        Err(e) => warn!("Field Electronics - Signal Error: {}", e)
                      }
                    }
                  },
                  JMSRole::Red => {
                    if let Some(stn) = stations.iter_mut().find(|x| x.id.alliance == Alliance::Red) {
                      let estop_state = invert ^ ep.status.cards[0].io_status[(stn.id.station - 1) * 2];
                      if estop_state != stn.physical_estop {
                        stn.set_physical_estop(estop_state, &self.kv)?;
                      }
                    }
                  },
                  JMSRole::Blue => {
                    if let Some(stn) = stations.iter_mut().find(|x| x.id.alliance == Alliance::Blue) {
                      let estop_state = invert ^ ep.status.cards[0].io_status[(stn.id.station - 1) * 2];
                      if estop_state != stn.physical_estop {
                        stn.set_physical_estop(estop_state, &self.kv)?;
                      }
                    }
                  },
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