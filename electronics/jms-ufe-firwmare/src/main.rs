// #![deny(warnings)]
#![no_main]
// #![no_std]
#![cfg_attr(not(test), no_std)]

use modbus::{ModbusTCPFrameRequest, Unpackable, ModbusTCPFrameResponse, Packable};
use panic_semihosting as _;
use stm32h7xx_hal::adc::{Adc, self};
use stm32h7xx_hal::device::{ADC1, DAC};
use stm32h7xx_hal::{gpio, dac};

use core::sync::atomic::AtomicU32;

use smoltcp::iface::{
    Interface, InterfaceBuilder, Neighbor, NeighborCache, Route, Routes,
    SocketStorage, SocketHandle,
};
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, Ipv6Cidr};
use smoltcp::socket::{TcpSocketBuffer, TcpSocket, UdpSocketBuffer, UdpPacketMetadata, UdpSocket};

use stm32h7xx_hal::{ethernet, rcc::CoreClocks, stm32};

fn systick_init(syst: &mut stm32::SYST, clocks: CoreClocks) {
    let c_ck_mhz = clocks.c_ck().to_MHz();

    let syst_calib = 1000;

    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload((syst_calib * c_ck_mhz) - 1);
    syst.enable_interrupt();
    syst.enable_counter();
}

/// TIME is an atomic u32 that counts milliseconds.
static TIME: AtomicU32 = AtomicU32::new(0);

const TCP_BUF_SIZE: usize = 4096;
const UDP_META_SIZE: usize = 24;
const UDP_BUF_SIZE: usize = 4096;

/// Ethernet descriptor rings are a global singleton
#[link_section = ".sram3.eth"]
static mut DES_RING: ethernet::DesRing<4, 4> = ethernet::DesRing::new();

// TODO: Add pull down / pull up state
#[derive(Default)]
pub struct IOStates {
  digital_out: [bool; 4],
  digital_in: [bool; 4],
  analog_in: [u16; 4],
  analog_out: [u16; 2],
  /* TODO: PWM */
}

/// Net storage with static initialisation - another global singleton
pub struct NetStorageStatic<'a> {
    ip_addrs: [IpCidr; 1],
    socket_storage: [SocketStorage<'a>; 8],
    neighbor_cache_storage: [Option<(IpAddress, Neighbor)>; 8],
    routes_storage: [Option<(IpCidr, Route)>; 1],

    modbus_buf: ([u8; TCP_BUF_SIZE], [u8; TCP_BUF_SIZE]),
    dmx_udp_buf: ([UdpPacketMetadata; UDP_META_SIZE], [u8; UDP_BUF_SIZE], [UdpPacketMetadata; UDP_META_SIZE], [u8; UDP_BUF_SIZE]),
}
static mut STORE: NetStorageStatic = NetStorageStatic {
    // Garbage
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

    /// Polls on the ethernet interface. You should refer to the smoltcp
    /// documentation for poll() to understand how to call poll efficiently
    pub fn poll(&mut self, now: i64, io: &mut IOStates) {
        let timestamp = Instant::from_millis(now);

        self.iface
            .poll(timestamp)
            .map(|_| ())
            .unwrap();

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

        {
          let dmx_socket: &mut UdpSocket = self.iface.get_socket(self.dmx_socket_handle);

          if !dmx_socket.is_open() {
            dmx_socket.bind(8283).unwrap();
          }

          if dmx_socket.can_recv() {
            let (data, _endpoint) = dmx_socket.recv().unwrap();
            
          }
        }
    }
}

pub struct IODevices {
  digital_in: [gpio::ErasedPin<gpio::Input>; 4],
  digital_out: [gpio::ErasedPin<gpio::Output<gpio::PushPull> >; 4],
  adc1: Adc<ADC1, adc::Enabled>,
  analog_in: (gpio::gpioa::PA3<gpio::Analog>, gpio::gpioc::PC0<gpio::Analog>, gpio::gpiob::PB1<gpio::Analog>, gpio::gpioa::PA6<gpio::Analog>),
  analog_out: (dac::C1<DAC, dac::Enabled>, dac::C2<DAC, dac::Enabled>)
}

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
mod app {
    use stm32h7xx_hal::{ethernet, ethernet::PHY, gpio, adc, prelude::*, signature::Uid, delay::Delay, rcc::rec::AdcClkSel};
    use stm32h7xx_hal::traits::DacOut;

    use super::*;
    use core::sync::atomic::Ordering;

    #[shared]
    struct SharedResources {
      io: IOStates
    }

    #[local]
    struct LocalResources {
        net: Net<'static>,
        devices: IODevices
        // lan8742a: ethernet::phy::LAN8742A<ethernet::EthernetMAC>,
    }

    #[init]
    fn init(
        mut ctx: init::Context,
    ) -> (SharedResources, LocalResources, init::Monotonics) {
        // Initialise power...
        let pwr = ctx.device.PWR.constrain();
        let pwrcfg = pwr.freeze();

        // Link the SRAM3 power state to CPU1
        ctx.device.RCC.ahb2enr.modify(|_, w| w.sram3en().set_bit());

        // Initialise clocks...
        let rcc = ctx.device.RCC.constrain();
        let ccdr = rcc
            .sys_ck(100.MHz())
            .hclk(100.MHz())
            .pll2_p_ck(50.MHz())
            .freeze(pwrcfg, &ctx.device.SYSCFG);

        // Initialise system...
        ctx.core.SCB.enable_icache();
        ctx.core.DWT.enable_cycle_counter();

        // Initialise IO...
        let gpioa = ctx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = ctx.device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = ctx.device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = ctx.device.GPIOG.split(ccdr.peripheral.GPIOG);

        let mut led = gpiob.pb0.into_push_pull_output();
        led.set_low();

        let rmii_ref_clk = gpioa.pa1.into_alternate();
        let rmii_mdio = gpioa.pa2.into_alternate();
        let rmii_mdc = gpioc.pc1.into_alternate();
        let rmii_crs_dv = gpioa.pa7.into_alternate();
        let rmii_rxd0 = gpioc.pc4.into_alternate();
        let rmii_rxd1 = gpioc.pc5.into_alternate();
        let rmii_tx_en = gpiog.pg11.into_alternate();
        let rmii_txd0 = gpiog.pg13.into_alternate();
        let rmii_txd1 = gpiob.pb13.into_alternate();

        /* Generate MAC Address */
        let uid = Uid::read();
        let mut hash: u32 = 0;
        for b in uid {
          hash = (hash.wrapping_mul(31)) ^ (*b as u32);
        }
        let hash_bytes = hash.to_le_bytes();
        let mac: [u8; 6] = [0x02, 0x00, hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]];

        // Init Ethernet
        let mac_addr = smoltcp::wire::EthernetAddress::from_bytes(&mac);
        let (eth_dma, _eth_mac) = unsafe {
            ethernet::new(
                ctx.device.ETHERNET_MAC,
                ctx.device.ETHERNET_MTL,
                ctx.device.ETHERNET_DMA,
                (
                    rmii_ref_clk,
                    rmii_mdio,
                    rmii_mdc,
                    rmii_crs_dv,
                    rmii_rxd0,
                    rmii_rxd1,
                    rmii_tx_en,
                    rmii_txd0,
                    rmii_txd1,
                ),
                &mut DES_RING,
                mac_addr,
                ccdr.peripheral.ETH1MAC,
                &ccdr.clocks,
            )
        };

        unsafe { ethernet::enable_interrupt() };

        // unsafe: mutable reference to static storage, we only do this once
        let store = unsafe { &mut STORE };
        let net = Net::new(store, eth_dma, mac_addr.into(), TIME.load(Ordering::Relaxed) as i64);

        let mut delay = Delay::new(ctx.core.SYST, ccdr.clocks);
        
        let mut adc1 = adc::Adc::adc1(
          ctx.device.ADC1,
          4.MHz(),
          &mut delay,
          ccdr.peripheral.ADC12,
          &ccdr.clocks
        ).enable();
        adc1.set_resolution(adc::Resolution::SixteenBit);
        
        let (dac1, dac2) = ctx.device.DAC.dac((gpioa.pa4, gpioa.pa5), ccdr.peripheral.DAC12);
        let dac1 = dac1.calibrate_buffer(&mut delay).enable();
        let dac2 = dac2.calibrate_buffer(&mut delay).enable();
        
        led.set_high();

        // Delay to show LED status
        delay.delay_ms(1000u16);

        // 1ms tick
        systick_init(&mut delay.free(), ccdr.clocks);

        (
            SharedResources {
              io: IOStates::default()
            },
            LocalResources {
                net,
                devices: IODevices {
                  digital_in: [
                    gpiof.pf3.into_pull_up_input().erase(),
                    gpiod.pd15.into_pull_up_input().erase(),
                    gpiod.pd14.into_pull_up_input().erase(),
                    gpiob.pb5.into_pull_up_input().erase()
                  ],
                  digital_out: [
                    led.erase(),
                    gpiob.pb14.into_push_pull_output().erase(),
                    gpioe.pe1.into_push_pull_output().erase(),
                    gpioe.pe0.into_push_pull_output().erase()
                  ],
                  adc1,
                  analog_in: (
                    gpioa.pa3.into_analog(),
                    gpioc.pc0.into_analog(),
                    gpiob.pb1.into_analog(),
                    gpioa.pa6.into_analog()
                  ),
                  analog_out: (
                    dac1, dac2
                  )
                }
            },
            init::Monotonics(),
        )
    }

    #[idle(shared = [io], local = [devices])]
    fn idle(mut ctx: idle::Context) -> ! {
        loop {
          let devices = &mut ctx.local.devices;
          ctx.shared.io.lock(|io| {
            for i in 0..devices.digital_in.len() {
              io.digital_in[i] = devices.digital_in[i].is_high();
            }
            for i in 0..devices.digital_out.len() {
              devices.digital_out[i].set_state(io.digital_out[i].into());
            }

            let mut val: u32 = devices.adc1.read(&mut devices.analog_in.0).unwrap_or(0);
            io.analog_in[0] = val as u16;
            val = devices.adc1.read(&mut devices.analog_in.1).unwrap_or(0);
            io.analog_in[1] = val as u16;
            val = devices.adc1.read(&mut devices.analog_in.2).unwrap_or(0);
            io.analog_in[2] = val as u16;
            val = devices.adc1.read(&mut devices.analog_in.3).unwrap_or(0);
            io.analog_in[3] = val as u16;

            devices.analog_out.0.set_value(io.analog_out[0].clamp(0, 4095));
            devices.analog_out.1.set_value(io.analog_out[1].clamp(0, 4095));
          });
        }
    }

    #[task(binds = ETH, shared = [io], local = [net])]
    fn ethernet_event(mut ctx: ethernet_event::Context) {
        unsafe { ethernet::interrupt_handler() }

        let time = TIME.load(Ordering::Relaxed);
        // TODO: Only lock IO if there's a message on ethernet that requires it
        let net = ctx.local.net;
        ctx.shared.io.lock(|io| {
          net.poll(time as i64, io);
        });
    }

    #[task(binds = SysTick, priority=15)]
    fn systick_tick(_: systick_tick::Context) {
        TIME.fetch_add(1, Ordering::Relaxed);
    }
}