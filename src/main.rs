#![allow(unused_imports)]
#![no_main]
#![no_std]


#[macro_use]
use lazy_static;

use panic_halt;

use hal::prelude::*;
// use f3::hal::prelude::*;
use cortex_m;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let _y;
    let x = 42;
    _y = x;
    
    // Setup serial
    
    // Setup Delay
    
    // Setup ITM
    
    // Setup GPIO

    // infinite loop; just so we don't leave this stack frame
    loop {}
}
