#![allow(unused_imports)]
#![no_main]
#![no_std]

// #[macro_use]
// use lazy_static;

use panic_itm; // use panic_halt; -> print's panics to itm
use cortex_m::{self, iprint, iprintln, peripheral::ITM};
use cortex_m_rt::{entry, exception, ExceptionFrame};

// #[macro_use(block)]
// extern crate nb;

use hal;
use hal::prelude::*;
use hal::delay::{self, Delay};
use hal::stm32::{GPIOB, GPIOA, ADC1, GPIOE, ADC2, GPIOC};
use hal::gpio::{*, gpioa::*, gpiob::*, gpioc::*};

use hal::adc::*;

use max7219::*;
use max7219::connectors::*;

use uecosystem::snake::*;
use uecosystem::joystick::*;

type CONNECTOR = PinConnector<PB8<Output<PushPull>>, PB9<Output<PushPull>>, PB10<Output<PushPull>>>;


fn initialise() -> (Delay, ITM, PB7<Input<PullDown>>, MAX7219<CONNECTOR>, Joystick)
{
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // PB7 -> Input from motion sensor
    let mut gpiob       = dp.GPIOB.split(&mut rcc.ahb);
    let motion_sensor   = gpiob.pb7.into_pull_down_input(&mut gpiob.moder, &mut gpiob.pupdr);

    // SPI1 for display (PA4=NSS, PA5=SCK, PA6=MISO, PA7=MOSI)
    // let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);
    // let mut gpioc   = dp.GPIOC.split(&mut rcc.ahb);
    let data        = gpiob.pb8.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let cs          = gpiob.pb9.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let sck         = gpiob.pb10.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let display     = MAX7219::from_pins(1, data, cs, sck).unwrap();

    let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);

    let joystick = Joystick {
        adc_x: Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
        adc_y: Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
        x: gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
        y: gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
        switch: gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
    };

    // let joystick = Joystick::new();



    // Set up ADC1
    // let adc1 = Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks);
    // let adc2 = Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks);
    // // let adc1_in1_pin = gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    // // let adc2_in1_pin = gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    // let adc12_inx_pin = gpioc.pc0.into_analog(&mut gpioc.moder, &mut gpioc.pupdr);
    // let adc12_in8_pin = gpioc.pc1.into_analog(&mut gpioc.moder, &mut gpioc.pupdr);
    // let switch = gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);




    (Delay::new(cp.SYST, clocks), cp.ITM, motion_sensor, display, joystick)
}

fn _wait_for_motion(sensor: &PB7<Input<PullDown>>) {
    while sensor.is_low().unwrap() {}
}


#[entry]
fn main() -> ! {
    let (mut delay, mut itm, _motion_sensor, mut display, mut joystick) = initialise();

    let mut gameworld = Game::new();

    // display.power_off().unwrap();
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);

    // make sure to wake the display up
    display.power_on().unwrap();
    // set display intensity lower
    // display.set_intensity(0, 0x1).unwrap();

    iprintln!(&mut itm.stim[0], "[WARN] Attempting to use the motion sensor before 60s elapsed may result in undefined behaviour");

    let mut counter: u8 = 0;
    loop {
        // gameworld.tick();
        //iprintln!(&mut itm.stim[0], "[{}] motion detected - {:?}", counter, motion_sensor.is_high().unwrap());

        // let readx: i16 =  as i16;
        // let ready: i16 =  as i16;
        // let x_sample: u16 = adc.read(&mut x).unwrap();

        iprintln!(&mut itm.stim[0], "joystick: direction={:?} {:?}, switch={}", joystick.raw_xy(), joystick.direction(), joystick.is_pressed().unwrap());

        let matrix: [u8; 8] = [
            counter,
            ((counter as u16 + 1) % 255) as u8,
            ((counter as u16 + 2) % 255) as u8,
            ((counter as u16 + 3) % 255) as u8,
            ((counter as u16 + 4) % 255) as u8,
            ((counter as u16 + 5) % 255) as u8,
            ((counter as u16 + 6) % 255) as u8,
            ((counter as u16 + 7) % 255) as u8,
        ];

        match display.write_raw(0, &matrix) {
            Err(_) => iprintln!(&mut itm.stim[0], "[ERROR] Refreshing display failed"),
            _ => (),
        }

        counter = (counter + 1) % 255;
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


#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
