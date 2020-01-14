#![allow(unused_imports)]
#![no_main]
#![no_std]

// #[macro_use]
// use lazy_static;

use panic_halt;
use cortex_m::{self, iprint, iprintln, peripheral::ITM};
use cortex_m_rt::entry;

use hal;
use hal::prelude::*;
use hal::delay::{self, Delay};
use hal::device::{GPIOB, GPIOA};
use hal::gpio::*;

use max7219::*;
// use max7219::connectors::*;

fn initialise() -> (Delay, ITM, hal::gpio::PB7<PullDown, Input>) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::device::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // PB7 -> Input from motion sensor
    let gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // SPI1 for display (PA4=NSS, PA5=SCK, PA6=MISO, PA7=MOSI)
    let gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let data = gpioa.pa6.alternating(hal::gpio::AF5);
    let sck = gpioa.pa5.alternating(hal::gpio::AF5);
    let cs = gpioa.pa4.alternating(hal::gpio::AF5);

    let _display = MAX7219::from_pins(1, data, cs, sck).unwrap();


    // gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    // let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    //
    // let


    // let data = pin!(gpio, spi0_mosi).into_output();
    // let sck = pin!(gpio, spi0_sck).into_output();
    // let cs = pin!(gpio, spi0_ss0).into_output();
    //
    // let mut display = MAX7219::from_pins(1, data, cs, sck).unwrap();


    (Delay::new(cp.SYST, clocks), cp.ITM, gpiob.pb7.input().pull_type(hal::gpio::PullDown))
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm, motion_sensor) = initialise();

    // TODO: Replace ITM with serial
    iprintln!(&mut itm.stim[0], "[WARN] Attempting to use the motion sensor before 60s elapsed may result in undefined behaviour");

    let mut counter = 0;
    loop {
        iprintln!(&mut itm.stim[0], "[{}] motion detected - {:?}", counter, motion_sensor.is_high().unwrap());
        counter += 1;
        delay.delay_ms(200_u16);
    }
}
