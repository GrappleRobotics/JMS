use smoltcp::iface::{
    Interface, InterfaceBuilder, Neighbor, NeighborCache, Route, Routes,
    SocketStorage, SocketHandle,
};
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, Ipv6Cidr};
use smoltcp::socket::{TcpSocketBuffer, TcpSocket, UdpSocketBuffer, UdpPacketMetadata, UdpSocket};

use stm32h7xx_hal::ethernet;

use modbus::{ModbusTCPFrameRequest, Unpackable, ModbusTCPFrameResponse, Packable};

use crate::IOStates;

const TCP_BUF_SIZE: usize = 4096;
const UDP_META_SIZE: usize = 24;
const UDP_BUF_SIZE: usize = 4096;

pub struct NetStorageStatic<'a> {
  ip_addrs: [IpCidr; 1],
  socket_storage: [SocketStorage<'a>; 8],
  neighbor_cache_storage: [Option<(IpAddress, Neighbor)>; 8],
  routes_storage: [Option<(IpCidr, Route)>; 1],

  modbus_buf: ([u8; TCP_BUF_SIZE], [u8; TCP_BUF_SIZE]),
  dmx_udp_buf: ([UdpPacketMetadata; UDP_META_SIZE], [u8; UDP_BUF_SIZE], [UdpPacketMetadata; UDP_META_SIZE], [u8; UDP_BUF_SIZE]),
}

pub static mut STORE: NetStorageStatic = NetStorageStatic {
  ip_addrs: [IpCidr::Ipv6(Ipv6Cidr::SOLICITED_NODE_PREFIX)],
  socket_storage: [SocketStorage::EMPTY; 8],
  neighbor_cache_storage: [None; 8],
  routes_storage: [None; 1],

  modbus_buf: ([0; TCP_BUF_SIZE], [0; TCP_BUF_SIZE]),
  dmx_udp_buf: ([UdpPacketMetadata::EMPTY; UDP_META_SIZE], [0; UDP_BUF_SIZE], [UdpPacketMetadata::EMPTY; UDP_META_SIZE], [0; UDP_BUF_SIZE])
};

pub struct Net<'a> {
  iface: Interface<'a, ethernet::EthernetDMA<'a, 4, 4>>,
  modbus_socket_handle: SocketHandle,
  dmx_socket_handle: SocketHandle
}
impl<'a> Net<'a> {
  pub fn new(
    store: &'static mut NetStorageStatic<'a>,
    ethdev: ethernet::EthernetDMA<'a, 4, 4>,
    ethernet_addr: HardwareAddress,
    _now: i64,
  ) -> Self {
    // Set IP address
    store.ip_addrs = [IpCidr::new(IpAddress::v4(10, 1, 10, 155), 24)];

    let neighbor_cache =
      NeighborCache::new(&mut store.neighbor_cache_storage[..]);
    let routes = Routes::new(&mut store.routes_storage[..]);

    let mut iface =
      InterfaceBuilder::new(ethdev, &mut store.socket_storage[..])
        .hardware_addr(ethernet_addr)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut store.ip_addrs[..])
        .routes(routes)
        .finalize();

    let modbus_rx_buffer = TcpSocketBuffer::new(&mut store.modbus_buf.0[..]);
    let modbus_tx_buffer = TcpSocketBuffer::new(&mut store.modbus_buf.1[..]);
    let modbus_socket = TcpSocket::new(modbus_rx_buffer, modbus_tx_buffer);

    let dmx_udp_rx_buffer = UdpSocketBuffer::new(&mut store.dmx_udp_buf.0[..], &mut store.dmx_udp_buf.1[..]);
    let dmx_udp_tx_buffer = UdpSocketBuffer::new(&mut store.dmx_udp_buf.2[..], &mut store.dmx_udp_buf.3[..]);
    let dmx_socket = UdpSocket::new(dmx_udp_rx_buffer, dmx_udp_tx_buffer);

    Net { modbus_socket_handle: iface.add_socket(modbus_socket), dmx_socket_handle: iface.add_socket(dmx_socket), iface }
  }

  pub fn poll(&mut self, now: i64, io: &mut IOStates) {
    let timestamp = Instant::from_millis(now);

    self.iface
      .poll(timestamp)
      .map(|_| ())
      .unwrap();

    // Modbus
    {
      let modbus_socket: &mut TcpSocket = self.iface.get_socket(self.modbus_socket_handle);

      if !modbus_socket.is_open() {
        modbus_socket.listen(502).unwrap();
      }

      if modbus_socket.may_recv() {
        let data = modbus_socket.recv(|buffer| {
          let len = buffer.len();
          (len, ModbusTCPFrameRequest::unpack(&buffer[..len]))
        }).unwrap();

        match data {
          Ok(req) => {
            let response = match req.function {
              modbus::ModbusFunctionRequest::ReadDiscreteInputs { first_address, n } => {
                modbus::ModbusFunctionResponse::ReadDiscreteInputs(
                  io.digital_in.get(first_address as usize .. (first_address + n) as usize).ok_or(modbus::ModbusExceptionCode::IllegalDataAddress)
                )
              },
              modbus::ModbusFunctionRequest::ReadCoils { first_address, n } => {
                modbus::ModbusFunctionResponse::ReadCoils(
                  io.digital_out.get(first_address as usize .. (first_address + n) as usize).ok_or(modbus::ModbusExceptionCode::IllegalDataAddress)
                )
              },
              modbus::ModbusFunctionRequest::WriteCoil { address, value } => {
                modbus::ModbusFunctionResponse::WriteCoil(
                  match io.digital_out.get_mut(address as usize) {
                    Some(v) => { *v = value; Ok((address, value)) },
                    None => Err(modbus::ModbusExceptionCode::IllegalDataAddress)
                  }
                )
              },
              modbus::ModbusFunctionRequest::WriteCoils { first_address, n, values } => {
                modbus::ModbusFunctionResponse::WriteCoils(
                  match io.digital_out.get_mut(first_address as usize .. (first_address + n) as usize) {
                    Some(v) => { v.clone_from_slice(&values[0..n as usize]); Ok((first_address, n)) },
                    None => Err(modbus::ModbusExceptionCode::IllegalDataAddress)
                  }
                )
              },
              modbus::ModbusFunctionRequest::ReadInputRegisters { first_address, n } => {
                modbus::ModbusFunctionResponse::ReadInputRegisters(
                  io.analog_in.get(first_address as usize .. (first_address + n) as usize).ok_or(modbus::ModbusExceptionCode::IllegalDataAddress)
                )
              },
              modbus::ModbusFunctionRequest::ReadHoldingRegisters { first_address, n } => {
                modbus::ModbusFunctionResponse::ReadHoldingRegisters(
                  io.analog_out.get(first_address as usize .. (first_address + n) as usize).ok_or(modbus::ModbusExceptionCode::IllegalDataAddress)
                )
              },
              modbus::ModbusFunctionRequest::WriteHoldingRegister { address, value } => {
                modbus::ModbusFunctionResponse::WriteHoldingRegister(
                  match io.analog_out.get_mut(address as usize) {
                    Some(v) => { *v = value; Ok((address, value)) },
                    None => Err(modbus::ModbusExceptionCode::IllegalDataAddress)
                  }
                )
              },
              modbus::ModbusFunctionRequest::WriteHoldingRegisters { first_address, n, values } => {
                modbus::ModbusFunctionResponse::WriteHoldingRegisters(
                  match io.analog_out.get_mut(first_address as usize .. (first_address + n) as usize) {
                    Some(v) => { v.clone_from_slice(&values[0..n as usize]); Ok((first_address, n)) },
                    None => Err(modbus::ModbusExceptionCode::IllegalDataAddress)
                  }
                )
              },
            };

            if modbus_socket.can_send() {
              let full_packet = ModbusTCPFrameResponse {
                transaction_id: req.transaction_id,
                protocol_id: req.protocol_id,
                unit_id: req.unit_id,
                function: response,
              };

              let mut buf = [0u8; 256];
              if let Ok(n) = full_packet.pack(&mut buf) {
                modbus_socket.send_slice(&buf[0..n]).unwrap();
              }
            }
          },
          Err(_) => ()  /* TODO */
        }
      } else if modbus_socket.may_send() {
        modbus_socket.close();
      }
    }

    // DMX
    {
      let dmx_socket: &mut UdpSocket = self.iface.get_socket(self.dmx_socket_handle);

      if !dmx_socket.is_open() {
        dmx_socket.bind(8283).unwrap();
      }

      if dmx_socket.can_recv() {
        let (data, _endpoint) = dmx_socket.recv().unwrap();
        let len = data.len().clamp(0, 512);
        (&mut io.dmx[0..len]).clone_from_slice(&data[0..len]);
      }
    }
  }
}
