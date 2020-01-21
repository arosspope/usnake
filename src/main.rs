#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_itm;
use cortex_m::{self, iprintln, peripheral::ITM};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    prelude::*,
    delay::Delay,
    // gpio::{*, gpiob::*},
    time::MonoTimer,
    adc::*
};

use max7219::{*};
use usnake::{game::*, joystick::*};

fn initialise() -> (Delay, ITM, Joystick, Game)
{
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Display pins
    let mut gpiob   = dp.GPIOB.split(&mut rcc.ahb);
    let data        = gpiob.pb8.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let cs          = gpiob.pb9.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let sck         = gpiob.pb10.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let display     = MAX7219::from_pins(1, data, cs, sck).expect("Unable to initialise display");

    let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);
    let joystick    = Joystick::from_pins(
        Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
        Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
        gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
        gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
        gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
    ).expect("Unable to initialise joystick class");

    let game = Game::new(
        MonoTimer::new(cp.DWT, clocks).now(),
        display
    );

    // Set up ADC1
    // let adc1 = Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks);
    // let adc2 = Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks);
    // // let adc1_in1_pin = gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    // // let adc2_in1_pin = gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);
    // let adc12_inx_pin = gpioc.pc0.into_analog(&mut gpioc.moder, &mut gpioc.pupdr);
    // let adc12_in8_pin = gpioc.pc1.into_analog(&mut gpioc.moder, &mut gpioc.pupdr);
    // let switch = gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);




    (Delay::new(cp.SYST, clocks), cp.ITM, joystick, game)
}

#[entry]
fn main() -> ! {
    let (mut delay, mut itm, mut joystick, mut gameworld) = initialise();

    // display.power_off().unwrap();
    // wait_for_motion(&motion_sensor);
    // delay.delay_ms(1000_u16);

    // make sure to wake the display up
    // gameworld.display.power_on().unwrap();
    // set display intensity lower
    // display.set_intensity(0, 0x1).unwrap();

    // iprintln!(&mut itm.stim[0], "[WARN] Attempting to use the motion sensor before 60s elapsed may result in undefined behaviour");
    loop {
        let dir = joystick.direction().expect("Unable to read from joystick");
        // iprintln!(&mut itm.stim[0], "joystick: direction={:?}, switch={}", dir, joystick.is_pressed().unwrap());
        if !gameworld.tick(dir){
            break;
        }
        //iprintln!(&mut itm.stim[0], "[{}] motion detected - {:?}", counter, motion_sensor.is_high().unwrap());

        // let readx: i16 =  as i16;
        // let ready: i16 =  as i16;
        // let x_sample: u16 = adc.read(&mut x).unwrap();

        // Operation ~19ms

        // let elapsed = instant.elapsed();
        // iprintln!(&mut itm.stim[0], "elapsed ({}us)", elapsed as f32 / timer.frequency().0 as f32 * 1e6);
        delay.delay_ms(200_u16);
    }

    let mut counter: u8 = 0;
    gameworld.display.power_on().expect("Unable to turn on display");
    loop {
        let matrix: [u8; 8] = [
            counter,
            ((counter as u16 + 1) % 256) as u8,
            ((counter as u16 + 2) % 256) as u8,
            ((counter as u16 + 3) % 256) as u8,
            ((counter as u16 + 4) % 256) as u8,
            ((counter as u16 + 5) % 256) as u8,
            ((counter as u16 + 6) % 256) as u8,
            ((counter as u16 + 7) % 256) as u8,
        ];

        match gameworld.display.write_raw(0, &matrix) {
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
