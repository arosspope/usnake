#![allow(unused_imports)]


#![no_main]
#![no_std]


use panic_itm;
use cortex_m::{self, iprintln, itm, peripheral::ITM};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    prelude::*,
    delay::Delay,
    time::MonoTimer,
    adc::*,
    stm32::{Peripherals, interrupt, Interrupt}
};

use max7219::*;
use usnake::{game::*, joystick::*};

// fn initialise() -> (Delay, ITM, Game)
// {
//     let cp = cortex_m::Peripherals::take().unwrap();
//     let mut dp = hal::stm32::Peripherals::take().unwrap();
//
//     let mut flash = dp.FLASH.constrain();
//     let mut rcc = dp.RCC.constrain();
//     let clocks = rcc.cfgr.freeze(&mut flash.acr);
//
//     // Display pins
//     let mut gpiob   = dp.GPIOB.split(&mut rcc.ahb);
//     let data        = gpiob.pb8.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
//     let cs          = gpiob.pb9.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
//     let sck         = gpiob.pb10.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
//     let display     = MAX7219::from_pins(1, data, cs, sck).expect("Unable to initialise display");
//
//     let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);
//     let joystick    = Joystick::from_pins(
//         Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
//         Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
//         gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
//         gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
//         gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
//     ).expect("Unable to initialise joystick class");
//
//     let controller = Game::new(
//         MonoTimer::new(cp.DWT, clocks).now(),
//         display,
//         joystick
//     );
//
//     (Delay::new(cp.SYST, clocks), cp.ITM, controller)
// }
//
// fn screensaver(count: u8) -> [u8; 8] {
//     let display: [u8; 8] = [
//         count,
//         ((count as u16 + 1) % 256) as u8,
//         ((count as u16 + 2) % 256) as u8,
//         ((count as u16 + 3) % 256) as u8,
//         ((count as u16 + 4) % 256) as u8,
//         ((count as u16 + 5) % 256) as u8,
//         ((count as u16 + 6) % 256) as u8,
//         ((count as u16 + 7) % 256) as u8,
//     ];
//
//     display
// }
//
//
// fn wait_for_user(delay: &mut Delay, controller: &mut Game) {
//     controller.display.power_on().expect("error with display");
//
//     let mut count = 0;
//     loop {
//         // While waiting for user input display screensaver
//         if controller.joystick.is_pressed().expect("error with joystick"){
//             break;
//         }
//
//         controller.display.write_raw(0, &screensaver(count)).expect("could not write to display");
//         count = (count + 1) % 255;
//         delay.delay_ms(100_u16);
//     }
// }
//
// fn usnake(delay: &mut Delay, controller: &mut Game) -> usize {
//     loop {
//         if let Some(score) = controller.tick() {
//             // Game complete!
//             for _ in 0..7 {
//                 controller.display.power_off().expect("error with display");
//                 delay.delay_ms(500_u16);
//                 controller.display.power_on().expect("error with display");
//                 delay.delay_ms(500_u16);
//             }
//             controller.reset();
//             return score
//         };
//
//         delay.delay_ms(200_u16);
//     }
// }
//
// #[entry]
// fn main() -> ! {
//     let (mut delay, mut itm, mut controller) = initialise();
//
//     // display.power_off().unwrap();
//     // wait_for_motion(&motion_sensor);
//     // delay.delay_ms(1000_u16);
//
//     // make sure to wake the display up
//     // gameworld.display.power_on().unwrap();
//     // set display intensity lower
//     // display.set_intensity(0, 0x1).unwrap();
//
//     loop {
//         iprintln!(&mut itm.stim[0], "waiting for user input....");
//         wait_for_user(&mut delay, &mut controller);
//
//         iprintln!(&mut itm.stim[0], "game start...");
//         let score = usnake(&mut delay, &mut controller);
//         iprintln!(&mut itm.stim[0], "game end - final score: {}", score);
//     }
//
//
//
//     // TODO:
//     //  [] Use RTFM to orchestrate the game
//     //      - https://rtfm.rs/0.5/book/en/preface.html
//     //      - https://github.com/rnestler/hello-rtfm-rs
// }
use core::fmt::{self, Write};

/// Macro for sending a formatted string through an ITM channel, with a newline.
macro_rules! logprint {
    ($logger:expr, $s:expr) => {
        $logger.lock(|logger: &mut ITM| {
            itm::write_fmt(&mut logger.stim[0], $s);
        });
    };
    ($logger:expr, $($arg:tt)*) => {
        $logger.lock(|logger: &mut ITM| {
            itm::write_fmt(&mut logger.stim[0], format_args!($($arg)*));
        });
    };
}

macro_rules! logprintln {
    ($logger:expr) => {
        logprint!($logger, "\n");
    };
    ($logger:expr, $fmt:expr) => {
        logprint!($logger, format_args!(concat!($fmt, "\n")));
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*) => {
        logprint!($logger, concat!($fmt, "\n"), $($arg)*);
    };
}



use rtfm::{app, Exclusive, Mutex};

#[app(device = hal::stm32, peripherals = true)]
const SNAKE: () = {
    // Late resources
    struct Resources {
        logger: ITM,
        controller: Game
    }

    #[init(spawn = [foo])]
    fn init(context: init::Context) -> init::LateResources {
        let mut core        : cortex_m::Peripherals     = context.core;         // Cortex-M peripherals
        let mut device  : hal::stm32::Peripherals   = context.device;   // Device specific peripherals

        let mut flash   = device.FLASH.constrain();
        let mut rcc     = device.RCC.constrain();
        let clocks      = rcc.cfgr.freeze(&mut flash.acr);

        // Setup for the MAX7219 display
        let mut gpiob   = device.GPIOB.split(&mut rcc.ahb);
        let data        = gpiob.pb8.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let cs          = gpiob.pb9.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let sck         = gpiob.pb10.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
        let display     = MAX7219::from_pins(1, data, cs, sck).expect("Unable to initialise display");

        // Setup Joystick
        let mut gpioa   = device.GPIOA.split(&mut rcc.ahb);
        let joystick    = Joystick::from_pins(
            Adc::adc1(device.ADC1, &mut device.ADC1_2, &mut rcc.ahb, clocks),
            Adc::adc2(device.ADC2, &mut device.ADC1_2, &mut rcc.ahb, clocks),
            gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
        ).expect("Unable to initialise joystick class");

        // Initialise game controller
        let controller = Game::new(
            MonoTimer::new(core.DWT, clocks).now(),
            display,
            joystick
        );

        // Spawn foo thread
        context.spawn.foo().unwrap();
        iprintln!(&mut core.ITM.stim[0], "... finished initialisation ...");

        init::LateResources { logger: core.ITM, controller: controller }
    }

    #[idle(resources = [logger])]
    fn idle(mut context: idle::Context) -> ! {
        // The idle task gets directly executed after init and should not return
        // RTFM will substitute its own handler if this is not declared
        logprintln!(&mut context.resources.logger, "... hit the idle task ...");

        loop {}
    }

    #[task(spawn = [bar, baz], resources = [logger])]
    fn foo(mut context: foo::Context) {
        logprintln!(&mut context.resources.logger, "foo - start");
        //
        // // spawns `bar` onto the task scheduler
        // // `foo` and `bar` have the same priority so `bar` will not run until
        // // after `foo` terminates
        context.spawn.bar().unwrap();
        //
        logprintln!(&mut context.resources.logger, "foo - middle");
        //
        // // spawns `baz` onto the task scheduler
        // // `baz` has higher priority than `foo` so it immediately preempts `foo`
        context.spawn.baz().unwrap();
        //
        logprintln!(&mut context.resources.logger, "foo - end");
    }

    #[task(resources = [logger])]
    fn bar(mut context: bar::Context) {
        logprintln!(&mut context.resources.logger, "bar");
    }

    #[task(priority = 2, resources = [logger])]
    fn baz(context: baz::Context) {
        logprintln!(Exclusive(context.resources.logger), "baz");
    }

    // Interrupt handlers used to dispatch software tasks
    // Software tasks can also be assigned priorities and, under the hood, are dispatched from interrupt handlers.
    // RTFM requires that free interrupts are declared in an 'extern block' when using software tasks;
    // some of these free interrupts will be used to dispatch the software tasks. An advantage of software
    // tasks over hardware tasks is that many tasks can be mapped to a single interrupt handler.
    extern "C" {
        fn DMA1_CH1();
        fn DMA1_CH2();
    }

};

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
