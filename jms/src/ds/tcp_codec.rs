use bitvec::prelude::*;
use bytes::{Buf, BufMut, Bytes};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio_util::codec::{Decoder, Encoder};

use crate::{arena::station::AllianceStationId, utils::bufutil::utf8_str_with_len};

#[derive(Debug)]
pub struct Ds2FmsTCP {
  pub tags: Vec<Ds2FmsTCPTags>,
}

#[derive(Debug, Display)]
pub enum Ds2FmsTCPTags {
  TeamNumber(u16),
  WPILibVersion(String),
  RIOVersion(String),
  DSVersion(String),
  LogData(DSLogData),
  Ping,
  ErrorData(Vec<TimestampedMessage>),
  Unknown(u8, usize),
}

#[derive(Debug)]
pub struct TimestampedMessage {
  timestamp: DateTime<Utc>,
  message: String,
}

// This is based off dslog
// https://github.com/ligerbots/dslogparser/blob/master/dslogparser/dslogparser.py
#[derive(Default, Debug)]
pub struct DSLogData {
  pub rtt: f64,
  pub lost_percent: f64,
  pub battery: f64,

  // Status
  pub brownout: bool,
  pub watchdog: bool,
  pub ds_teleop: bool,
  pub ds_auto: bool,
  pub ds_disable: bool,
  pub robot_teleop: bool,
  pub robot_auto: bool,
  pub robot_disable: bool,

  pub rio_cpu: f64,
  pub can_usage: f64,
  pub wifi_db: f64,
  pub bandwidth: f64,
  pub pdp: Bytes,
}

#[derive(Debug)]
pub struct Fms2DsTCP {
  pub tags: Vec<Fms2DsTCPTags>,
}

#[derive(Debug)]
pub enum Fms2DsTCPTags {
  #[allow(dead_code)]
  EventCode(String),
  StationInfo(AllianceStationId, Fms2DsStationStatus),
  #[allow(dead_code)]
  GameData(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Fms2DsStationStatus {
  Good = 0,
  Bad = 1,
  Waiting = 2,
}

pub struct DSTCPCodec {
  decode_frame_len: u16,
}

impl DSTCPCodec {
  pub fn new() -> DSTCPCodec {
    DSTCPCodec { decode_frame_len: 0 }
  }
}

impl Decoder for DSTCPCodec {
  type Item = Ds2FmsTCP;
  type Error = std::io::Error;

  // Frames come in as:
  // Size, ID, Tags...
  // Each TCP packet can have multiple frames (multiple tag classes), but each frame's
  // ID is the same for all following tags. The tag list may have multiple tags of the
  // same type, such as in big LogData chunks.
  // For an example, the first packet the DS sends is the Team Number.
  // The second packet is a ping (0x1d), the DS Version, one LogData, and an ErrorData.

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if self.decode_frame_len == 0 {
      // Haven't got a length yet. Note this length is for the current tag.
      if src.remaining() < 2 {
        return Ok(None);
      } else {
        self.decode_frame_len = src.get_u16();
        return self.decode(src);
      }
    } else {
      // Currently decoding a frame
      if src.remaining() < self.decode_frame_len.into() {
        return Ok(None);
      }

      let mut buf = src.split_to(self.decode_frame_len as usize);

      // Have whole frame - decode
      let id = buf.get_u8();
      let mut pkt = Ds2FmsTCP { tags: vec![] };
      let mut tags_ok = true;
      while buf.has_remaining() && tags_ok {
        let size = buf.remaining(); // Full size of current tag class (including all tags)
        let tag: Option<Ds2FmsTCPTags> = match id {
          0x00 => {
            // WPILib Version
            Some(Ds2FmsTCPTags::WPILibVersion(utf8_str_with_len(&mut buf, size)?))
          }
          0x01 => {
            // RIO Version
            Some(Ds2FmsTCPTags::RIOVersion(utf8_str_with_len(&mut buf, size)?))
          }
          0x02 => {
            // DS Version
            Some(Ds2FmsTCPTags::DSVersion(utf8_str_with_len(&mut buf, size)?))
          }
          0x16 => {
            // Log Data
            let mut dat: DSLogData = Default::default();
            dat.rtt = (buf.get_u8() as f64) / 2.0; // TODO: Is this /2 or /15
            dat.lost_percent = (buf.get_u8() as f64) * 4.0;

            {
              let ones = buf.get_u8();
              let decs = buf.get_u8();
              dat.battery = (ones as f64) + (decs as f64) / 256.0;
            }

            dat.rio_cpu = (buf.get_u8() as f64) / 2.0;

            {
              let status = !buf.get_u8();
              let bv = status.view_bits::<Msb0>();
              dat.brownout = bv[0];
              dat.watchdog = bv[1];
              dat.ds_teleop = bv[2];
              dat.ds_auto = bv[3];
              dat.ds_disable = bv[4];
              dat.robot_teleop = bv[5];
              dat.robot_auto = bv[6];
              dat.robot_disable = bv[7];
            }

            dat.can_usage = (buf.get_u8() as f64) / 2.0;
            dat.wifi_db = (buf.get_u8() as f64) / 2.0;
            dat.bandwidth = (buf.get_u16() as f64) / 256.0;

            // Unknown
            buf.get_u8();

            // PDP - TODO
            dat.pdp = buf.copy_to_bytes(24);

            Some(Ds2FmsTCPTags::LogData(dat))
          }
          0x17 => {
            // Error / events data
            let count = buf.get_u32();
            let msgs: Result<Vec<TimestampedMessage>, Self::Error> = (0..count)
              .map(|_| {
                let secs_since_1904 = buf.get_u64() as i64;
                /* Offset to the unix epoch */
                let datetime = NaiveDateTime::from_timestamp(secs_since_1904 - 2_082_844_800, 0);

                buf.get_u64(); // Unknown bytes
                let msg_len = buf.get_u32();

                Ok(TimestampedMessage {
                  timestamp: DateTime::<Utc>::from_utc(datetime, Utc),
                  message: utf8_str_with_len(&mut buf, msg_len as usize)?,
                })
              })
              .collect();

            Some(Ds2FmsTCPTags::ErrorData(msgs?))
          }
          0x18 => {
            // Team Number
            Some(Ds2FmsTCPTags::TeamNumber(buf.get_u16()))
          }
          0x1c => Some(Ds2FmsTCPTags::Ping),
          0x1d => {
            buf.get_u8(); // Don't know why but this has an extra '0' almost always
            Some(Ds2FmsTCPTags::Ping)
          }
          unknown => {
            tags_ok = false;
            Some(Ds2FmsTCPTags::Unknown(unknown, buf.remaining()))
          }
        };
        if let Some(t) = tag {
          pkt.tags.push(t);
        }
      }

      self.decode_frame_len = 0;
      Ok(Some(pkt))
    }
  }
}

impl Encoder<Fms2DsTCP> for DSTCPCodec {
  type Error = std::io::Error;

  fn encode(&mut self, pkt: Fms2DsTCP, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    for tag in pkt.tags {
      // Buffer the writer since we need the size before the rest of the data
      let mut writer: Vec<u8> = vec![];

      match tag {
        Fms2DsTCPTags::EventCode(code) => {
          writer.put_u8(0x14);
          writer.put_u8(code.len() as u8);
          writer.extend_from_slice(code.as_bytes());
        }
        Fms2DsTCPTags::StationInfo(stn, status) => {
          writer.put_u8(0x19);
          writer.put_u8(stn.into());
          writer.put_u8(status as u8);
        }
        Fms2DsTCPTags::GameData(data) => {
          writer.put_u8(0x1c);
          writer.put_u8(data.len() as u8);
          writer.extend_from_slice(data.as_bytes());
        }
      }

      dst.reserve(1);
      dst.put_u16(writer.len() as u16);
      dst.extend_from_slice(&writer);
    }

    Ok(())
  }
}
