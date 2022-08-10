pub mod comms;
pub mod service;
pub mod settings;

use bitvec::{prelude::Lsb0, view::BitView};

use crate::arena::lighting::{LightMode, Colour};

use self::comms::{Unpackable, Packable};

#[derive(strum_macros::FromRepr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElectronicsRole {
  JMS = 0,
  ScoringTable = 1,
  BlueDs = 2,
  RedDs = 3
}

impl Packable for ElectronicsRole {
  fn pack(&self, buf: &mut dyn bytes::BufMut) {
    buf.put_u8(*self as u8);
  }
}

impl Unpackable for ElectronicsRole {
  fn unpack(buf: &mut dyn bytes::Buf) -> Self {
    return ElectronicsRole::from_repr(buf.get_u8()).unwrap();
  }
}

#[derive(Debug, Clone)]
pub struct EstopStates {
  field: bool,
  red: [bool; 3],
  blue: [bool; 3]
}

impl Unpackable for EstopStates {
  fn unpack(buf: &mut dyn bytes::Buf) -> Self {
    let byte = buf.get_u8();
    let bits = byte.view_bits::<Lsb0>();
    return Self { field: bits[0], red: [bits[1], bits[2], bits[3]], blue: [bits[4], bits[5], bits[6]] };
  }
}

impl Packable for Colour {
  fn pack(&self, buf: &mut dyn bytes::BufMut) {
    buf.put_u8(self.red);
    buf.put_u8(self.green);
    buf.put_u8(self.blue);
  }
}

impl Packable for LightMode {
  fn pack(&self, buf: &mut dyn bytes::BufMut) {
    match self {
      LightMode::Off => buf.put_u8(0),
      LightMode::Constant(colour) => {
        buf.put_u8(1);
        colour.pack(buf);
      },
      LightMode::Pulse(colour, duration) => {
        buf.put_u8(2);
        colour.pack(buf);
        buf.put_u16(duration.num_milliseconds() as u16);
      },
      LightMode::Chase(colour, duration) => {
        buf.put_u8(3);
        colour.pack(buf);
        buf.put_u16(duration.num_milliseconds() as u16);
      },
      LightMode::Rainbow(duration) => {
        buf.put_u8(4);
        buf.put_u16(duration.num_milliseconds() as u16);
      },
    }
  }
}

#[derive(Debug, Clone)]
pub enum ElectronicsMessageOut {
  Ping,
  SetLights(Vec<LightMode>),
}

impl Packable for ElectronicsMessageOut {
  fn pack(&self, buf: &mut dyn bytes::BufMut) {
    match self {
      ElectronicsMessageOut::Ping => buf.put_u8(0),
      ElectronicsMessageOut::SetLights(modes) => {
        buf.put_u8(2);
        buf.put_u8(modes.len() as u8);
        for mode in modes {
          mode.pack(buf);
        }
      },
    }
  }
}

#[derive(Debug, Clone)]
pub struct AddressedElectronicsMessageOut{
  role: ElectronicsRole,
  msg: ElectronicsMessageOut
}

impl Packable for AddressedElectronicsMessageOut {
  fn pack(&self, buf: &mut dyn bytes::BufMut) {
    self.role.pack(buf);
    self.msg.pack(buf);
  }
}

#[derive(Debug, Clone)]
pub enum ElectronicsMessageIn {
  Ping,
  Estop(EstopStates)
}

impl Unpackable for ElectronicsMessageIn {
  fn unpack(buf: &mut dyn bytes::Buf) -> Self {
    let id = buf.get_u8();
    match id {
      0 => ElectronicsMessageIn::Ping,
      1 => ElectronicsMessageIn::Estop(Unpackable::unpack(buf)),
      _ => panic!("Unknown ID") // TODO: Better error handling here
    }
  }
}

#[derive(Debug, Clone)]
pub struct AddressedElectronicsMessageIn {
  #[allow(dead_code)]
  role: ElectronicsRole,
  msg: ElectronicsMessageIn
}

impl Unpackable for AddressedElectronicsMessageIn {
  fn unpack(buf: &mut dyn bytes::Buf) -> Self {
    Self {
      role: ElectronicsRole::unpack(buf),
      msg: ElectronicsMessageIn::unpack(buf)
    }
  }
}