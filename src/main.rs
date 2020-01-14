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
use hal::device::GPIOB;
use hal::gpio::*;

fn initialise() -> (Delay, ITM, hal::gpio::PB7<PullDown, Input>) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::device::Peripherals::take().unwrap();
    
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    
    // PB7 -> Input from motion sensor
    let gpiob = dp.GPIOB.split(&mut rcc.ahb);
    
    (Delay::new(cp.SYST, clocks), cp.ITM, gpiob.pb7.input().pull_type(hal::gpio::PullDown))
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm, motion_sensor) = initialise();
    
    // delay.delay_ms(60 * 1000_u16);
    
    // TODO: Replace ITM with serial
    let mut counter = 0;
    loop {
        iprintln!(&mut itm.stim[0], "{}s: {:?}", counter, motion_sensor.is_high().unwrap());
        counter += 1;
        delay.delay_ms(200_u16);
    }
}
