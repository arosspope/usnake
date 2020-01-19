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
use hal::stm32::{GPIOB, GPIOA};
use hal::gpio::{*, gpioa::*, gpiob::*};

use max7219::*;
use max7219::connectors::*;

use uecosystem::snake::*;

type CONNECTOR = PinConnector<PA6<Output<PushPull>>, PA4<Output<PushPull>>, PA5<Output<PushPull>>>;


fn initialise() -> (Delay, ITM, PB7<Input<PullDown>>, MAX7219<CONNECTOR>)
{
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // PB7 -> Input from motion sensor
    let mut gpiob       = dp.GPIOB.split(&mut rcc.ahb);
    let motion_sensor   = gpiob.pb7.into_pull_down_input(&mut gpiob.moder, &mut gpiob.pupdr);

    // SPI1 for display (PA4=NSS, PA5=SCK, PA6=MISO, PA7=MOSI)
    let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);
    let data        = gpioa.pa6.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let sck         = gpioa.pa5.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let cs          = gpioa.pa4.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let display     = MAX7219::from_pins(1, data, cs, sck).unwrap();

    (Delay::new(cp.SYST, clocks), cp.ITM, motion_sensor, display)
}

fn _wait_for_motion(sensor: &PB7<Input<PullDown>>) {
    while sensor.is_low().unwrap() {}
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm, _motion_sensor, mut display) = initialise();

    let mut gameworld = Game::new();

    // display.power_off().unwrap();
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);

    // make sure to wake the display up
    display.power_on().unwrap();
    // set display intensity lower
    // display.set_intensity(0, 0x1).unwrap();

    iprintln!(&mut itm.stim[0], "[WARN] Attempting to use the motion sensor before 60s elapsed may result in undefined behaviour");

    let mut counter = 0;
    loop {
        gameworld.tick();
        // iprintln!(&mut itm.stim[0], "[{}] motion detected - {:?}", counter, motion_sensor.is_high().unwrap());

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

        counter += 1;
        delay.delay_ms(150_u16);
    }

    // TODO:
    //  [] Read joystick controller input (joystick.rs) - will require use of ADC
    //      - https://github.com/stm32-rs/stm32f3xx-hal/pull/47
    //      - Could maybe turn it into an embeded-hal abstraction crate?
    //  [] Implement display class - For updating the world every tick of the game world
    //  [] Use RTFM to orchestrate the game
    //      - https://rtfm.rs/0.5/book/en/preface.html
    //      - https://github.com/rnestler/hello-rtfm-rs
}


// use cortex_m_rt::{entry, exception, ExceptionFrame};
// #[exception]
// fn HardFault(ef: &ExceptionFrame) -> ! {
//     panic!("HardFault at {:#?}", ef);
// }
//
// #[exception]
// fn DefaultHandler(irqn: i16) {
//     panic!("Unhandled exception (IRQn = {})", irqn);
// }
