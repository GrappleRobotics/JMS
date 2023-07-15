#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use stm32f0xx_hal::pac;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use stm32f0xx_hal as hal;
use hal::prelude::*;
use hal::{pac::interrupt, rcc::{RccExt, USBClockSource}, usb::{Peripheral, UsbBus}};

#[entry]
fn main() -> ! {
  let mut dp = pac::Peripherals::take().unwrap();
  let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();

  let mut rcc = dp
    .RCC
    .configure()
    .sysclk(48.mhz())
    .usbsrc(USBClockSource::PLL)
    .freeze(&mut dp.FLASH);

  let gpioa = dp.GPIOA.split(&mut rcc);
  let gpiob = dp.GPIOB.split(&mut rcc);
  let (mut led, pin_dm, mut pin_dp, estop) = cortex_m::interrupt::free(|cs| (gpiob.pb9.into_push_pull_output(cs), gpioa.pa11, gpioa.pa12.into_push_pull_output(cs), gpioa.pa7.into_pull_up_input(cs)));
  led.set_low().unwrap();

  // Blip dp low to signal a new device
  let mut delay = hal::delay::Delay::new(cp.SYST, &rcc);
  pin_dp.set_low().unwrap();
  delay.delay_ms(500u32);

  let pin_dp = cortex_m::interrupt::free(|cs| pin_dp.into_floating_input(cs));

  let usb = Peripheral {
      usb: dp.USB,
      pin_dm,
      pin_dp,
  };

  let usb_bus = UsbBus::new(usb);

  let mut serial = SerialPort::new(&usb_bus);

  let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x7171, 0x7172))
    .manufacturer("Grapple Robotics")
    .product("JMS Scoring Table Hardware")
    .serial_number("ABCD")
    .device_class(USB_CLASS_CDC)
    .build();

  loop {
    if !usb_dev.poll(&mut [&mut serial]) {
        continue;
    }

    let mut buf = [0u8; 64];

    match serial.read(&mut buf) {
      Ok(count) if count > 0 => {
        led.set_high().unwrap();

        let mut status = [ b'-' ];
        if estop.is_high().unwrap() {
          status[0] = b'E';
        }

        serial.write(&status[..]).unwrap();

        // TODO: DMX here
      }
      _ => {}
    }

    led.set_low().unwrap();
  }
}
