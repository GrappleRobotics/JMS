// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;
use stm32h7xx_hal::gpio;

use core::sync::atomic::AtomicU32;

use smoltcp::iface::{
    Interface, InterfaceBuilder, Neighbor, NeighborCache, Route, Routes,
    SocketStorage, SocketHandle,
};
use smoltcp::time::Instant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr, Ipv6Cidr};
use smoltcp::socket::{TcpSocketBuffer, TcpSocket, UdpSocketBuffer, UdpPacketMetadata, UdpSocket};

use stm32h7xx_hal::{ethernet, rcc::CoreClocks, stm32};

fn systick_init(mut syst: stm32::SYST, clocks: CoreClocks) {
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
    pub fn poll(&mut self, now: i64) {
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
              
              let mut data = [0; 2048];
              for i in 0..len {
                data[i] = buffer[i];
              }

              (len, (len, data))
            }).unwrap();

            if modbus_socket.can_send() && data.0 > 0 {
              modbus_socket.send_slice(&data.1[0..data.0]).unwrap();
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

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
mod app {
    use stm32h7xx_hal::{ethernet, ethernet::PHY, gpio, prelude::*, signature::Uid};

    use super::*;
    use core::sync::atomic::Ordering;

    #[shared]
    struct SharedResources {}
    #[local]
    struct LocalResources {
        net: Net<'static>,
        // lan8742a: ethernet::phy::LAN8742A<ethernet::EthernetMAC>,
        link_led: gpio::gpiob::PB0<gpio::Output<gpio::PushPull>>,
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
            .freeze(pwrcfg, &ctx.device.SYSCFG);

        // Initialise system...
        ctx.core.SCB.enable_icache();
        ctx.core.DWT.enable_cycle_counter();

        // Initialise IO...
        let gpioa = ctx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiog = ctx.device.GPIOG.split(ccdr.peripheral.GPIOG);
        // let gpioi = ctx.device.GPIOI.split(ccdr.peripheral.GPIOI);
        let mut link_led = gpiob.pb0.into_push_pull_output(); // LED3
        link_led.set_low();

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

        let mac_addr = smoltcp::wire::EthernetAddress::from_bytes(&mac);
        let (eth_dma, eth_mac) = unsafe {
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

        // Initialise ethernet PHY...
        // let mut lan8742a = ethernet::phy::LAN8742A::new(eth_mac);
        // lan8742a.phy_reset();
        // lan8742a.phy_init();
        // The eth_dma should not be used until the PHY reports the link is up

        unsafe { ethernet::enable_interrupt() };

        // unsafe: mutable reference to static storage, we only do this once
        let store = unsafe { &mut STORE };
        let net = Net::new(store, eth_dma, mac_addr.into(), TIME.load(Ordering::Relaxed) as i64);

        // 1ms tick
        systick_init(ctx.core.SYST, ccdr.clocks);

        (
            SharedResources {},
            LocalResources {
                net,
                // lan8742a,
                link_led,
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [])]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            // Ethernet
            // match ctx.local.lan8742a.poll_link() {
            //     true => ctx.local.link_led.set_high(),
            //     _ => ctx.local.link_led.set_low(),
            // }
        }
    }

    #[task(binds = ETH, local = [net, link_led])]
    fn ethernet_event(ctx: ethernet_event::Context) {
        unsafe { ethernet::interrupt_handler() }

        ctx.local.link_led.set_low();
        let time = TIME.load(Ordering::Relaxed);
        ctx.local.net.poll(time as i64);
        ctx.local.link_led.set_high();
    }

    #[task(binds = SysTick, priority=15)]
    fn systick_tick(_: systick_tick::Context) {
        TIME.fetch_add(1, Ordering::Relaxed);
    }
}