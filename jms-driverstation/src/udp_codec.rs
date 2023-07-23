use bytes::{Buf, BufMut};
use chrono::{DateTime, Local, Timelike, Datelike};
use jms_core_lib::models::{AllianceStationId, MatchSubtype};
use jms_driverstation_lib::{TournamentLevel, RobotState};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Default, Debug, Clone)]
pub struct Ds2FmsUDP {
  pub seq: u16,
  pub team: u16,
  pub estop: bool,
  pub robot: bool,
  pub radio: bool,
  pub rio: bool,
  pub enabled: bool,
  pub mode: RobotState,
  pub battery: f64,
  pub tags: Vec<Ds2FmsUDPTags>,
}

#[derive(Debug, Clone)]
pub enum Ds2FmsUDPTags {
  FieldRadioMetrics(u8, u16), // Signal strength, Bandwidth Util
  CommsMetrics(u16, u16, u8), // Lost pkts, Sent pkts, Average trip time
  LaptopMetrics(u8, u8),      // Bat %, CPU %
  RobotRadioMetrics(u8, u16), // Signal strength, Bandwidth Util
  Unknown(u8, usize),
}

#[derive(Debug)]
pub struct Fms2DsUDP {
  pub estop: bool,
  pub enabled: bool,
  pub mode: RobotState,
  pub station: AllianceStationId,
  pub tournament_level: TournamentLevel,
  pub match_number: u16,
  pub play_number: u8,
  pub time: DateTime<Local>,
  pub remaining_seconds: u16,
}

pub struct DSUDPCodec {
  pub seq_num_enc: u16,
}

impl DSUDPCodec {
  pub fn new() -> DSUDPCodec {
    DSUDPCodec { seq_num_enc: 0 }
  }
}

impl Decoder for DSUDPCodec {
  type Item = Ds2FmsUDP;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.remaining() < 8 {
      return Ok(None);
    } else {
      let mut buf = src.split_to(src.remaining());

      let mut pkt: Ds2FmsUDP = Default::default();
      pkt.seq = buf.get_u16();
      buf.get_u8(); // Comms version
      {
        let status = buf.get_u8();
        pkt.estop = (status & 0b1000_0000) != 0;
        pkt.robot = (status & 0b0010_0000) != 0;
        pkt.radio = (status & 0b0001_0000) != 0;
        pkt.rio = (status & 0b0000_1000) != 0;
        pkt.enabled = (status & 0b0000_0100) != 0;
        pkt.mode = match status & 0b0000_0011 {
          0 => RobotState::Teleop,
          1 => RobotState::Test,
          _ => RobotState::Teleop
        }
      }
      pkt.team = buf.get_u16();
      {
        let ones = buf.get_u8();
        let decs = buf.get_u8();
        pkt.battery = (ones as f64) + (decs as f64) / 256.0;
      }

      while buf.has_remaining() {
        let size = (buf.get_u8() - 1) as usize;
        let id = buf.get_u8();
        let tag = match id {
          0x0 => {
            // Field radio
            let strength = buf.get_u8();
            let bandwidth = buf.get_u16();
            Ds2FmsUDPTags::FieldRadioMetrics(strength, bandwidth)
          }
          0x01 => {
            // Comms metrics
            let lost = buf.get_u16();
            let sent = buf.get_u16();
            let avg_tt = buf.get_u8();
            Ds2FmsUDPTags::CommsMetrics(lost, sent, avg_tt)
          }
          0x02 => {
            // Laptop metrics
            let batt = buf.get_u8();
            let cpu = buf.get_u8();
            Ds2FmsUDPTags::LaptopMetrics(batt, cpu)
          }
          0x03 => {
            // Radio metrics
            let strength = buf.get_u8();
            let bandwidth = buf.get_u16();
            Ds2FmsUDPTags::RobotRadioMetrics(strength, bandwidth)
          }
          unknown => {
            let _ = buf.split_to(size);
            Ds2FmsUDPTags::Unknown(unknown, size)
          }
        };
        pkt.tags.push(tag);
      }

      return Ok(Some(pkt));
    }
  }
}

impl Encoder<Fms2DsUDP> for DSUDPCodec {
  type Error = std::io::Error;

  fn encode(&mut self, pkt: Fms2DsUDP, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    let mut writer: Vec<u8> = vec![];

    self.seq_num_enc += 1;
    writer.put_u16(self.seq_num_enc);
    writer.put_u8(0x00);

    // Control
    let mode_bits = match pkt.mode {
      RobotState::Auto => 2,
      RobotState::Test => 1,
      RobotState::Teleop => 0,
    };
    let control: u8 = ((pkt.estop as u8) << 7) | ((pkt.enabled as u8) << 2) | (mode_bits & 0b11);
    writer.put_u8(control);

    writer.put_u8(0x00);
    writer.put_u8(pkt.station.to_ds_number());
    writer.put_u8(pkt.tournament_level as u8);
    writer.put_u16(pkt.match_number);
    writer.put_u8(pkt.play_number);

    // Date
    writer.put_u32(pkt.time.timestamp_subsec_micros());
    writer.put_u8(pkt.time.second() as u8);
    writer.put_u8(pkt.time.minute() as u8);
    writer.put_u8(pkt.time.hour() as u8);
    writer.put_u8(pkt.time.day() as u8);
    writer.put_u8(pkt.time.month() as u8);
    writer.put_u8((pkt.time.year() - 1900) as u8);

    writer.put_u16(pkt.remaining_seconds);

    // Tags - none so far

    dst.extend_from_slice(&writer);
    Ok(())
  }
}