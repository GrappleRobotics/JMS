#![cfg_attr(not(test), no_std)]

use core::convert::TryInto;
use core::result::Result;

#[derive(Debug)]
pub enum PackError {
  BufferOverrun,
  UnknownFunction
}

fn ensure_size(sz: usize, buf: &[u8]) -> Result<(), PackError> {
  if buf.len() < sz {
    Err(PackError::BufferOverrun)
  } else {
    Ok(())
  }
}

pub trait Packable : Sized {
  fn pack(&self, buf: &mut [u8]) -> Result<usize, PackError>;
}

pub trait Unpackable : Sized {
  fn unpack(buf: &[u8]) -> Result<Self, PackError>;
}

/* REQUEST */

#[derive(Debug)]
#[allow(dead_code)]
struct ModbusTCPFrameRequest {
  transaction_id: u16,
  protocol_id: u16,
  unit_id: u8,
  function: ModbusFunctionRequest
}

impl Unpackable for ModbusTCPFrameRequest {
  fn unpack(buf: &[u8]) -> Result<Self, PackError> {
    ensure_size(6, buf)?;
    let transaction_id = u16::from_be_bytes((&buf[0..=1]).try_into().unwrap());
    let protocol_id = u16::from_be_bytes((&buf[2..=3]).try_into().unwrap());
    let length = u16::from_be_bytes((&buf[4..=5]).try_into().unwrap());
    let unit_id = buf[6];

    ensure_size(length as usize + 6, buf)?;

    Ok(Self {
      transaction_id,
      protocol_id,
      unit_id,
      function: ModbusFunctionRequest::unpack(&buf[7..])?
    })
  }
}

const WRITE_COILS_MAX: usize = 128;
const WRITE_HOLDING_MAX: usize = 128;

#[derive(Debug)]
#[allow(dead_code)]
enum ModbusFunctionRequest {
  ReadDiscreteInputs { first_address: u16, n: u16 },
  ReadCoils { first_address: u16, n: u16 },
  WriteCoil { address: u16, value: bool },
  WriteCoils { first_address: u16, n: u16, values: [bool; WRITE_COILS_MAX] },
  ReadInputRegisters { first_address: u16, n: u16 },
  ReadHoldingRegisters { first_address: u16, n: u16 },
  WriteHoldingRegister { address: u16, value: u16 },
  WriteHoldingRegisters { first_address: u16, n: u16, values: [u16; WRITE_HOLDING_MAX] }
}

impl Unpackable for ModbusFunctionRequest {
  fn unpack(buf: &[u8]) -> Result<Self, PackError> {
    ensure_size(1, buf)?;
    let function_code = buf[0];

    let nbuf = &buf[1..];

    match function_code {
      0x01 => {
        // Read Coils
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::ReadCoils { 
          first_address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()), 
          n: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap())
        })
      },
      0x02 => {
        // Read Discrete Inputs
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::ReadDiscreteInputs { 
          first_address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()), 
          n: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap())
        })
      },
      0x03 => {
        // Read Multiple Holding Registers
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::ReadHoldingRegisters { 
          first_address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()), 
          n: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap())
        })
      },
      0x04 => {
        // Read Input Registers
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::ReadInputRegisters { 
          first_address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()), 
          n: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap())
        })
      },
      0x05 => {
        // Write Single Coil
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::WriteCoil {
          address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()),
          value: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap()) > 0
        })
      },
      0x06 => {
        // Write Single Holding Register
        ensure_size(4, nbuf)?;
        Ok(ModbusFunctionRequest::WriteHoldingRegister {
          address: u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap()),
          value: u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap())
        })
      },
      0xF => {
        // Write Multiple Coils
        ensure_size(5, nbuf)?;
        let first_address = u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap());
        let n = u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap());
        let n_bytes = nbuf[4];
        ensure_size(5 + n_bytes as usize, nbuf)?;

        let mut values = [false; WRITE_COILS_MAX];
        if n as usize > values.len() {
          Err(PackError::BufferOverrun)?;
        }

        if (n / 8) as usize > (n_bytes as usize) {
          Err(PackError::BufferOverrun)?;
        }

        for b in 0..n_bytes {
          for i in 0..8 {
            values[b as usize * 8 + i as usize] = (nbuf[5 + b as usize] & (1 << i)) > 0
          }
        }

        Ok(ModbusFunctionRequest::WriteCoils { 
          first_address,
          n, 
          values
        })
      },
      0x10 => {
        // Write Multiple Holding Registers
        ensure_size(5, nbuf)?;
        let first_address = u16::from_be_bytes((&nbuf[0..=1]).try_into().unwrap());
        let n = u16::from_be_bytes((&nbuf[2..=3]).try_into().unwrap());
        let n_bytes = nbuf[4];
        ensure_size(5 + n_bytes as usize, nbuf)?;

        if (n * 2) as usize != n_bytes as usize {
          Err(PackError::BufferOverrun)?;
        }

        let mut values = [0u16; WRITE_HOLDING_MAX];
        if n as usize > values.len() {
          Err(PackError::BufferOverrun)?;
        }

        for i in 0..n {
          values[i as usize] = u16::from_be_bytes((&nbuf[5+i as usize..=6+i as usize]).try_into().unwrap());
        }

        Ok(ModbusFunctionRequest::WriteHoldingRegisters {
          first_address,
          n,
          values
        })
      },
      _ => Err(PackError::UnknownFunction)
    }
  }
}

/* RESPONSE */
#[derive(Debug)]
#[allow(dead_code)]
struct ModbusTCPFrameResponse<'a> {
  transaction_id: u16,
  protocol_id: u16,
  unit_id: u8,
  function: ModbusFunctionResponse<'a>
}

impl<'a> Packable for ModbusTCPFrameResponse<'a> {
  fn pack(&self, buf: &mut [u8]) -> Result<usize, PackError> {
    buf[0..=1].clone_from_slice(self.transaction_id.to_be_bytes().as_slice());
    buf[2..=3].clone_from_slice(self.protocol_id.to_be_bytes().as_slice());
    buf[6] = self.unit_id;
    let size_fn = self.function.pack(&mut buf[7..])?;
    buf[4..=5].clone_from_slice((size_fn as u16 + 1).to_be_bytes().as_slice());

    Ok(7 + size_fn)
  }
}

#[derive(Debug)]
#[allow(dead_code)]
enum ModbusFunctionResponse<'a> {
  ReadDiscreteInputs(ModbusResponse<&'a[bool]>),
  ReadCoils(ModbusResponse<&'a[bool]>),

  WriteCoil(ModbusResponse<(u16, bool)>),   // address, value
  WriteCoils(ModbusResponse<(u16, u16)>),   // first_address, n

  ReadInputRegisters(ModbusResponse<&'a[u16]>),
  ReadHoldingRegisters(ModbusResponse<&'a[u16]>),

  WriteHoldingRegister(ModbusResponse<(u16, u16)>),     // address, value
  WriteHoldingRegisters(ModbusResponse<(u16, u16)>),     // first_address, n
}

type ModbusResponse<T> = Result<T, ModbusExceptionCode>;

#[derive(Debug, Clone)]
#[repr(u8)]
#[allow(dead_code)]
enum ModbusExceptionCode {
  IllegalFunction = 0x01,
  IllegalDataAddress = 0x02,
  IllegalDataValue = 0x03,
  ServerDeviceFailure = 0x04,
  Acknowledge = 0x05,
  ServerDeviceBusy = 0x06,
  NegativeAcknowledge = 0x07,
  MemoryParityError = 0x08,
  GatewayPathUnavailable = 0x10,
  GatewayTargetDeviceFailedToRespond = 0x11
}

impl<'a> Packable for ModbusFunctionResponse<'a> {
  fn pack(&self, buf: &mut [u8]) -> Result<usize, PackError> {
    match self {
      ModbusFunctionResponse::ReadDiscreteInputs(Ok(values)) => {
        let n_bytes = values.len() / 8 + if values.len() % 8 != 0 { 1 } else { 0 };
        ensure_size(2 + n_bytes, buf)?;
        buf[0] = 0x02;
        buf[1] = n_bytes as u8;
        for b in 0..n_bytes {
          buf[2 + b] = 0;
          for i in 0..8 {
            if let Some(v) = values.get(b*8 + i) {
              if *v {
                buf[2 + b] |= 1 << i;
              }
            }
          }
        }

        Ok(2 + n_bytes)
      },
      ModbusFunctionResponse::ReadDiscreteInputs(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x82;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::ReadCoils(Ok(values)) => {
        let n_bytes = values.len() / 8 + if values.len() % 8 != 0 { 1 } else { 0 };
        ensure_size(2 + n_bytes, buf)?;
        buf[0] = 0x01;
        buf[1] = n_bytes as u8;
        for b in 0..n_bytes {
          buf[2 + b] = 0;
          for i in 0..8 {
            if let Some(v) = values.get(b*8 + i) {
              if *v {
                buf[2 + b] |= 1 << i;
              }
            }
          }
        }

        Ok(2 + n_bytes)
      },
      ModbusFunctionResponse::ReadCoils(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x81;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::WriteCoil(Ok((address, value))) => {
        ensure_size(5, buf)?;
        buf[0] = 0x05;
        buf[1..=2].clone_from_slice(address.to_be_bytes().as_slice());
        buf[3..=4].clone_from_slice((if *value { 65280u16 } else { 0u16 }).to_be_bytes().as_slice());

        Ok(5)
      },
      ModbusFunctionResponse::WriteCoil(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x85;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::WriteCoils(Ok((first_address, n))) => {
        ensure_size(5, buf)?;
        buf[0] = 0x0F;
        buf[1..=2].clone_from_slice(first_address.to_be_bytes().as_slice());
        buf[3..=4].clone_from_slice(n.to_be_bytes().as_slice());

        Ok(5)
      },
      ModbusFunctionResponse::WriteCoils(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x8F;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::ReadInputRegisters(Ok(values)) => {
        ensure_size(2 + 2*values.len(), buf)?;
        buf[0] = 0x04;
        buf[1] = (values.len() * 2) as u8;
        for i in 0..values.len() {
          buf[2+2*i..=3+2*i].clone_from_slice(values[i].to_be_bytes().as_slice());
        }

        Ok(2 + 2*values.len())
      },
      ModbusFunctionResponse::ReadInputRegisters(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x84;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::ReadHoldingRegisters(Ok(values)) => {
        ensure_size(2 + 2*values.len(), buf)?;
        buf[0] = 0x03;
        buf[1] = (values.len() * 2) as u8;
        for i in 0..values.len() {
          buf[2+2*i..=3+2*i].clone_from_slice(values[i].to_be_bytes().as_slice());
        }

        Ok(2 + 2*values.len())
      },
      ModbusFunctionResponse::ReadHoldingRegisters(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x83;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::WriteHoldingRegister(Ok((address, value))) => {
        ensure_size(5, buf)?;
        buf[0] = 0x06;
        buf[1..=2].clone_from_slice(address.to_be_bytes().as_slice());
        buf[3..=4].clone_from_slice(value.to_be_bytes().as_slice());

        Ok(5)
      },
      ModbusFunctionResponse::WriteHoldingRegister(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x86;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
      ModbusFunctionResponse::WriteHoldingRegisters(Ok((first_address, n))) => {
        ensure_size(5, buf)?;
        buf[0] = 0x10;
        buf[1..=2].clone_from_slice(first_address.to_be_bytes().as_slice());
        buf[3..=4].clone_from_slice(n.to_be_bytes().as_slice());

        Ok(5)
      },
      ModbusFunctionResponse::WriteHoldingRegisters(Err(e)) => {
        ensure_size(2, buf)?;
        buf[0] = 0x80 | 0x10;
        buf[1] = e.clone() as u8;
        Ok(2)
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{ModbusTCPFrameRequest, Unpackable, Packable, ModbusFunctionResponse, ModbusTCPFrameResponse};

  #[test]
  fn test_unpack() {
    let bytes: [u8; 12] = [
      0x00, 0x04, 0x00, 0x00, 0x00, 0x06, 0x02, 0x04,
      0x00, 0x34, 0x00, 0x06
    ];

    println!("TEST {:?}", ModbusTCPFrameRequest::unpack(&bytes[..]))
  }

  #[test]
  fn test_pack() {
    let response = ModbusTCPFrameResponse {
      transaction_id: 0x1234,
      protocol_id: 0x4567,
      unit_id: 0x89,
      function: ModbusFunctionResponse::ReadCoils(Ok(&[false, false, true, true, false, true, true, false, true])),
    };

    let mut buf = [0u8; 256];
    let n = response.pack(&mut buf[..]).unwrap();

    println!("{:0x?}", &buf[0..n]);
  }
}