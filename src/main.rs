#![allow(unused_imports)]
#![no_main]
#![no_std]

// #[macro_use]
// use lazy_static;

use panic_halt;
use cortex_m::{self, iprint, iprintln, peripheral::ITM};
use cortex_m_rt::entry;

use hal::prelude::*;
use hal::delay::{self, Delay};
use hal::gpio::Gpiob;
use hal::device::GPIOB;

fn initialise() -> (Delay, ITM, GPIOB) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::device::Peripherals::take().unwrap();
    
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC;
    rcc.ahbenr.modify(|_, w| w.iopben().set_bit());
    
    
    let clocks = rcc.constrain().cfgr.freeze(&mut flash.acr);
    
    
    
    // rcc.ahb.modify(|_, w| w.iopben().set_bit());
    
    // PA7 -> Input from motion sensor
    let gpiob = &dp.GPIOB;
    gpiob.moder.modify(|_, w| w.moder7().input());
    gpiob.pupdr.modify(|_, w| w.pupdr7().pull_up());
    
    
    (Delay::new(cp.SYST, clocks), cp.ITM, dp.GPIOB)
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm, gpiob) = initialise();
    
    // TODO: Replace ITM with serial
    let mut counter = 0;
    loop {
        let detection = gpiob.idr.read().bits();//.idr7().bit_is_set();
        
        iprintln!(&mut itm.stim[0], "{}s: {:b}", counter, detection);
        counter += 1;
        delay.delay_ms(1000_u16);
    }
}
