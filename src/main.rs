#![allow(unused_imports)]
#![no_main]
#![no_std]

// #[macro_use]
// use lazy_static;

use panic_halt;
use cortex_m::{self, iprint, iprintln, peripheral::ITM};
use cortex_m_rt::entry;

// #[macro_use(block)]
// extern crate nb;

use hal;
use hal::prelude::*;
use hal::delay::{self, Delay};
use hal::device::{GPIOB, GPIOA};
use hal::gpio::*;

use max7219::*;
use max7219::connectors::*;


type CONNECTOR = PinConnector<PA6<PullNone, Output<PushPull, LowSpeed>>, PA4<PullNone, Output<PushPull, LowSpeed>>, PA5<PullNone, Output<PushPull, LowSpeed>>>;
// type CONNECTOR = PinConnector<PA6<PullUp, Output<PushPull, LowSpeed>>, PA4<PullUp, Output<PushPull, LowSpeed>>, PA5<PullUp, Output<PushPull, LowSpeed>>>;

// type CONNECTOR = PinConnector<PA6<PullNone, AltFn<AF5, PushPull, LowSpeed>>, PA4<PullNone, AltFn<AF5, PushPull, LowSpeed>>, PA5<PullNone, AltFn<AF5, PushPull, LowSpeed>>>;

fn initialise() -> (Delay, ITM, PB7<PullDown, Input>, MAX7219<CONNECTOR>)
{
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::device::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // PB7 -> Input from motion sensor
    let gpiob           = dp.GPIOB.split(&mut rcc.ahb);
    let motion_sensor   = gpiob.pb7.input().pull_type(hal::gpio::PullDown);

    // SPI1 for display (PA4=NSS, PA5=SCK, PA6=MISO, PA7=MOSI)
    let gpioa   = dp.GPIOA.split(&mut rcc.ahb);
    // let data    = gpioa.pa6.alternating(hal::gpio::AF5);
    // let sck     = gpioa.pa5.alternating(hal::gpio::AF5);
    // let cs      = gpioa.pa4.alternating(hal::gpio::AF5);
    let data       = gpioa.pa6.output();
    let sck        = gpioa.pa5.output();
    let cs         = gpioa.pa4.output();
    let display = MAX7219::from_pins(1, data, cs, sck).unwrap();

    (Delay::new(cp.SYST, clocks), cp.ITM, motion_sensor, display)
}

fn wait_for_motion(sensor: &PB7<PullDown, Input>) {
    while sensor.is_low().unwrap() {}
}


#[entry]
fn main() -> ! {
    // let (mut delay, mut itm, motion_sensor, _data, _cs, _sck) = initialise();
    let (mut delay, mut itm, motion_sensor, mut display) = initialise();

    // display.power_off().unwrap();
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);
    
    // make sure to wake the display up
    display.power_on().unwrap();
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);
    
    
    // write given octet of ASCII characters with dots specified by 3rd param bits
    // display.write_str(0, b"a a a a ", 0b1111111).unwrap();
    
    
    
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);
    display.test(0, false).unwrap();
    // set display intensity lower
    display.set_intensity(0, 0x1).unwrap();
    // display.set_decode_mode(0, DecodeMode::CodeBDigits7_0).unwrap();

    // TODO: Replace ITM with serial
    iprintln!(&mut itm.stim[0], "[WARN] Attempting to use the motion sensor before 60s elapsed may result in undefined behaviour");

    let mut counter = 0;
    loop {
        // iprintln!(&mut itm.stim[0], "[{}] motion detected - {:?}", counter, motion_sensor.is_high().unwrap());
        // display.clear_display(0).unwrap();
        
        let matrix: [u8; 8] = [
            counter % 255,
            (counter + 1) % 254,
            (counter + 2) % 254,
            (counter + 3) % 254,
            (counter + 4) % 254,
            (counter + 5) % 254,
            (counter + 6) % 254,
            (counter + 7) % 254,
        ];
        
        match display.write_raw(0, &matrix) {
            Err(_) => iprintln!(&mut itm.stim[0], "[ERROR] Refreshing display failed"),
            _ => (),
        }
        // display.write_str(0, b"........", 0b11010101).unwrap();
        
        counter += 1;
        delay.delay_ms(150_u16);
    }
}
