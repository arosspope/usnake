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

fn initialise() -> (Delay, ITM) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::device::Peripherals::take().unwrap();
    
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    
    (Delay::new(cp.SYST, clocks), cp.ITM)
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm) = initialise();
    
    // Setup serial
    
    // Setup GPIO

    // infinite loop; just so we don't leave this stack frame
    loop {
        iprintln!(&mut itm.stim[0], "Hello World!");
        delay.delay_ms(1000_u16);
    }
}
