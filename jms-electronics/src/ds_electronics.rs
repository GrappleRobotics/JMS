use std::{net::{IpAddr, Ipv4Addr}, time::Duration};
use bytes::Buf;
use futures_util::StreamExt;
use deku::{*, bitvec::BitSlice};

use jms_arena_lib::AllianceStation;
use jms_base::{kv, mq};
use jms_core_lib::{models::Alliance, db::Table};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, FramedRead};

pub struct DriverStationElectronics {
  pub alliance: Alliance,
  pub ip: IpAddr,
  kv: kv::KVConnection,
  mq: mq::MessageQueueChannel
}

impl DriverStationElectronics {
  pub fn new(alliance: Alliance, ip: Option<IpAddr>, kv: kv::KVConnection, mq: mq::MessageQueueChannel) -> Self {
    Self {
      alliance,
      ip: ip.unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 0, 100, if alliance == Alliance::Blue { 21 } else { 22 }))),
      kv, mq
    }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    let stream = tokio::time::timeout(Duration::from_millis(3000), TcpStream::connect(self.ip.to_string() + ":1071")).await.map_err(|_| anyhow::anyhow!("Connection Timed Out"))?.map_err(|e| anyhow::anyhow!("Connect error: {}", e))?;
    let mut framed = FramedRead::new(stream, GrappleTcpCodec {});

    let mut cache_interval = tokio::time::interval(Duration::from_millis(500));

    let mut stations = AllianceStation::sorted(&self.kv)?;

    loop {
      tokio::select! {
        _ = cache_interval.tick() => {
          stations = AllianceStation::all(&self.kv)?;
        },
        msg = framed.next() => match msg {
          Some(Ok(data)) => match data {
            StatusMessage::Io(iomsg) => {
              for stn in &mut stations {
                if stn.id.alliance == self.alliance {
                  if (!iomsg.digital[stn.id.station - 1]) != stn.physical_estop {
                    stn.set_physical_estop(!iomsg.digital[stn.id.station - 1], &self.kv)?;
                  }
                }
              }
            },
            StatusMessage::Network(net) => {
              for stn in &mut stations {
                if stn.id.alliance == self.alliance {
                  let link = matches!(net.ports[stn.id.station], PortStatus::LinkUp { .. });
                  if Some(link) != stn.ds_eth_ok {
                    stn.set_ds_eth_ok(Some(link), &self.kv)?;
                  }
                }
              }
            }
          },
          Some(Err(e)) => log::error!("Framed Error: {}", e),
          None => (),
        }
      }
    }
  }
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
#[deku(type = "u8")]
pub enum PortDuplexStatus {
  #[deku(id = "0")]
  Half,
  #[deku(id = "1")]
  Full,
  #[deku(id = "2")]
  Unknown
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
#[deku(type = "u8")]
pub enum PortStatus {
  #[deku(id = "0")]
  NoLink,
  #[deku(id = "1")]
  AutonegotiationInProgress,
  #[deku(id = "2")]
  LinkUp {
    speed: u16,
    duplex: PortDuplexStatus,
  }
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
#[deku(type = "u8")]
pub enum NetworkStatusFrameSpecific {
  #[deku(id = "0")]
  Flat,
  #[deku(id = "1")]
  Vlan,
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
pub struct IOStatusFrame {
  #[deku(bits = 1)]
  pub digital: [bool; 8],
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
#[deku(ctx = "api_index: u8", id = "api_index")]
pub enum StatusMessage {
  #[deku(id = "0")]
  Network(NetworkStatusFrame),

  #[deku(id = "1")]
  Io(IOStatusFrame)
}

#[derive(Debug, Clone, DekuRead, DekuWrite, PartialEq, Eq)]
pub struct NetworkStatusFrame {
  pub management: PortStatus,
  pub usb: PortStatus,
  pub ports: [PortStatus; 6],
  pub specific: NetworkStatusFrameSpecific
}

pub struct GrappleTcpCodec {}

impl Decoder for GrappleTcpCodec {
  type Item = StatusMessage;
  type Error = anyhow::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.len() < 2 {
      return Ok(None);
    }

    let mut len_bytes: [u8; 2] = [0u8; 2];
    len_bytes.copy_from_slice(&src[..2]);
    let length = u16::from_le_bytes(len_bytes) as usize;

    if src.len() < 2 + length {
      src.reserve(2 + length - src.len());
      return Ok(None);
    }

    let data = src[2..2+length].to_vec();
    src.advance(2 + length);

    let device_type = data[0+5];
    let manufacturer = data[1+5];
    let api_class = data[2+5];
    let api_index = data[3+5];
    let _device_id = data[4+5];

    if manufacturer == 0x06 && device_type == 0x0c && api_class == 0x01 {
      return StatusMessage::read(BitSlice::from_slice(&data[5+5..]), api_index).map(|x| Some(x.1)).map_err(|e| anyhow::anyhow!("Decode Error: {}", e))
    }
    return Ok(None)
    // GrappleTCPMessage::from_bytes((&data[..], 0)).map(|v| Some(v.1)).map_err(|e| anyhow::anyhow!("Decode Error: {}", e))
  }
}
