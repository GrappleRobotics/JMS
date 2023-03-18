// #![deny(warnings)]
#![no_main]
// #![no_std]
#![cfg_attr(not(test), no_std)]

mod net;

use panic_semihosting as _;
use stm32h7xx_hal::adc::{Adc, self};
use stm32h7xx_hal::device::{ADC1, DAC, USART2};
use stm32h7xx_hal::serial::Serial;
use stm32h7xx_hal::{gpio, dac};

use core::sync::atomic::AtomicU32;

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

/// Ethernet descriptor rings are a global singleton
#[link_section = ".sram3.eth"]
static mut DES_RING: ethernet::DesRing<4, 4> = ethernet::DesRing::new();

// TODO: Add pull down / pull up state
pub struct IOStates {
  digital_out: [bool; 4],
  digital_in: [bool; 4],
  analog_in: [u16; 4],
  analog_out: [u16; 2],
  /* TODO: PWM */
  dmx: [u8; 513]
}

impl Default for IOStates {
  fn default() -> Self {
    Self {
      digital_out: Default::default(),
      digital_in: Default::default(),
      analog_in: Default::default(),
      analog_out: Default::default(),
      dmx: [0u8; 513]
    }
  }
}

/// Net storage with static initialisation - another global singleton
pub struct IODevices {
  digital_in: [gpio::ErasedPin<gpio::Input>; 4],
  digital_out: [gpio::ErasedPin<gpio::Output<gpio::PushPull> >; 4],
  adc1: Adc<ADC1, adc::Enabled>,
  analog_in: (gpio::gpioa::PA3<gpio::Analog>, gpio::gpioc::PC0<gpio::Analog>, gpio::gpiob::PB1<gpio::Analog>, gpio::gpioa::PA6<gpio::Analog>),
  analog_out: (dac::C1<DAC, dac::Enabled>, dac::C2<DAC, dac::Enabled>),
}

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
mod app {
    use stm32h7xx_hal::delay::DelayFromCountDownTimer;
    use stm32h7xx_hal::serial;
    use stm32h7xx_hal::timer::Event;
    use stm32h7xx_hal::{ethernet, gpio, adc, prelude::*, signature::Uid, delay::Delay};
    use stm32h7xx_hal::traits::DacOut;

    use crate::net::STORE;

    use super::*;
    use core::sync::atomic::Ordering;

    #[shared]
    struct SharedResources {
      io: IOStates
    }

    #[local]
    struct LocalResources {
        net: net::Net<'static>,
        devices: IODevices,
        dmx_timer: stm32h7xx_hal::timer::Timer<stm32::TIM2>,  // Option so we can .take()
        dmx_delay: DelayFromCountDownTimer<stm32h7xx_hal::timer::Timer<stm32::TIM3>>,
        dmx: (Serial<USART2>, gpio::ErasedPin<gpio::Output<gpio::PushPull>>)
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

        let store = unsafe { &mut STORE };
        let net = net::Net::new(store, eth_dma, mac_addr.into(), TIME.load(Ordering::Relaxed) as i64);

        let mut delay = Delay::new(ctx.core.SYST, ccdr.clocks);
        
        // Initialise ADC
        let mut adc1 = adc::Adc::adc1(
          ctx.device.ADC1,
          4.MHz(),
          &mut delay,
          ccdr.peripheral.ADC12,
          &ccdr.clocks
        ).enable();
        adc1.set_resolution(adc::Resolution::SixteenBit);
        
        // Initialise DAC
        let (dac1, dac2) = ctx.device.DAC.dac((gpioa.pa4, gpioa.pa5), ccdr.peripheral.DAC12);
        let dac1 = dac1.calibrate_buffer(&mut delay).enable();
        let dac2 = dac2.calibrate_buffer(&mut delay).enable();
        
        // Initialise DMX512
        // let mut dmx_tx_pin = gpiod.pd5.into_push_pull_output().speed(gpio::Speed::VeryHigh);
        let dmx_tx_pin = gpiod.pd5.into_alternate();
        let dmx_rx_pin = gpiod.pd6.into_alternate();
        let mut dmx_rts_pin = gpiod.pd4.into_push_pull_output().speed(gpio::Speed::VeryHigh);

        dmx_rts_pin.set_low();

        let dmx_config = serial::config::Config::new(250000.bps())
          .parity_none()
          .stopbits(serial::config::StopBits::Stop2)
          .bitorder(serial::config::BitOrder::LsbFirst);

        let dmx_serial = ctx.device.USART2
          .serial((dmx_tx_pin, dmx_rx_pin), dmx_config, ccdr.peripheral.USART2, &ccdr.clocks)
          .unwrap();

        // Setup DMX timer
        let mut dmx_timer = ctx.device.TIM2.timer(
          15.Hz(),
          ccdr.peripheral.TIM2,
          &ccdr.clocks
        );
        dmx_timer.listen(Event::TimeOut);

        let dmx_delay_timer = ctx.device.TIM3.timer(
          1.Hz(),
          ccdr.peripheral.TIM3,
          &ccdr.clocks
        );
        let dmx_delay = DelayFromCountDownTimer::new(dmx_delay_timer);

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
                  ),
                },
                dmx: (dmx_serial, dmx_rts_pin.erase()),
                dmx_timer,
                dmx_delay
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

    #[task(binds = TIM2, shared = [io], local = [dmx_timer, dmx_delay, dmx])]
    fn tim2_tick(mut ctx: tim2_tick::Context) {
      let dmx_data = ctx.shared.io.lock(|io| {
        io.dmx.clone()
      });
      
      let (serial, de) = ctx.local.dmx;
      de.set_high();
      
      let delay = ctx.local.dmx_delay;

      let stolen = unsafe { stm32::Peripherals::steal().GPIOD };
      stolen.moder.modify(|_, w| w.moder5().output());

      // START (BREAK + MAB)
      stolen.bsrr.write(|w| w.br5().set_bit());
      delay.delay_us(100u16);
      stolen.bsrr.write(|w| w.bs5().set_bit());
      delay.delay_us(12u16);

      // Send slot data
      stolen.moder.modify(|_, w| w.moder5().alternate());
      stolen.bsrr.write(|w| w.br5().set_bit());
      serial.bwrite_all(&dmx_data[..]).unwrap();

      // Restore
      stolen.moder.modify(|_, w| w.moder5().output());
      stolen.bsrr.write(|w| w.bs5().set_bit());
      de.set_low();

      ctx.local.dmx_timer.clear_irq();
    }
}