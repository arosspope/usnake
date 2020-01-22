#![allow(unused_imports)]


#![no_main]
#![no_std]


use rtfm::cyccnt::Duration;
use rtfm::cyccnt::{Instant, U32Ext};
use panic_itm;
use cortex_m::{self, iprintln, itm, peripheral::ITM};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    prelude::*,
    delay::Delay,
    time::{Hertz, MonoTimer},
    adc::*,
    stm32::{Peripherals, interrupt, Interrupt}
};

use max7219::*;
use usnake::{game::*, joystick::*};

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

#[app(device = hal::stm32, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const SNAKE: () = {
    // Late resources
    struct Resources {
        logger: ITM,
        controller: Game,
        sysclk_hz: Hertz
    }

    #[init(spawn = [idlescreen])]
    fn init(context: init::Context) -> init::LateResources {
        let mut core    : rtfm::Peripherals         = context.core;         // Cortex-M peripherals
        let mut device  : hal::stm32::Peripherals   = context.device;   // Device specific peripherals

        let mut flash   = device.FLASH.constrain();
        let mut rcc     = device.RCC.constrain();
        let clocks      = rcc.cfgr.freeze(&mut flash.acr);

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

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

        iprintln!(&mut core.ITM.stim[0], "... app start ...");
        context.spawn.idlescreen().expect("something went wrong whilst spawning the 'idlescreen' task");

        init::LateResources { logger: core.ITM, controller: controller, sysclk_hz: clocks.sysclk() }
    }


    #[task(spawn = [gametick], resources = [logger, controller, sysclk_hz], schedule = [idlescreen])]
    fn idlescreen(context: idlescreen::Context) {
        static mut SCREEN: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        for i in 0..SCREEN.len() {
            SCREEN[i] = ((SCREEN[i] + 1) as u16 % 256) as u8 % 255;
        }

        context.resources.controller.display.write_raw(0, &SCREEN).expect("unable to refresh screen");

        if context.resources.controller.joystick.is_pressed().expect("error with joystick") {
            logprintln!(Exclusive(context.resources.logger), "... game start ...");
            // Start game...
            context.spawn.gametick().expect("something went wrong whilst spawning the 'gametick' task");
        } else {
            // Every 150ms idlescreen is refreshed
            let delay = ((150e-3 as f32 * context.resources.sysclk_hz.0 as f32) as u32).cycles();
            context.schedule.idlescreen(context.scheduled + delay).expect("unable to schedule the idlescreen task");
        }
    }

    #[task(spawn = [gameend], resources = [logger, controller, sysclk_hz], schedule = [gametick])]
    fn gametick(context: gametick::Context) {
        if let Some(score) = context.resources.controller.tick() {
            context.spawn.gameend().expect("something went wrong whilst spawning the 'gameend' task");
            logprintln!(Exclusive(context.resources.logger), "... game end - final score: {} ...", score);
        } else {
            let delay = ((250e-3 as f32 * context.resources.sysclk_hz.0 as f32) as u32).cycles(); // Every 250ms the game 'ticks'
            context.schedule.gametick(context.scheduled + delay).expect("unable to schedule the gametick task");
        }
    }

    #[task(spawn = [idlescreen], resources = [controller, sysclk_hz], schedule = [gameend])]
    fn gameend(context: gameend::Context) {
        static mut SCREEN_ON: bool = true;
        static mut FLASH_SCREEN_COUNT: u8 = 0;

        if *FLASH_SCREEN_COUNT < 7 {
            match *SCREEN_ON {
                true => context.resources.controller.display.power_off().expect("unable to toggle display"),
                false => context.resources.controller.display.power_on().expect("unable to toggle display"),
            }


            let delay = ((500e-3 as f32 * context.resources.sysclk_hz.0 as f32) as u32).cycles(); // Every 250ms the game 'ticks'
            context.schedule.gameend(context.scheduled + delay).expect("unable to schedule the gametick task");
            *FLASH_SCREEN_COUNT += 1;
            *SCREEN_ON = !*SCREEN_ON;
        } else {
            *SCREEN_ON = true;
            *FLASH_SCREEN_COUNT = 0;
            context.resources.controller.reset();
            context.resources.controller.display.power_on().expect("unable to toggle display");
            context.spawn.idlescreen().expect("something went wrong whilst spawning the 'idlescreen' task");
        }
    }

    // Interrupt handlers used to dispatch software tasks
    // Software tasks can also be assigned priorities and, under the hood, are dispatched from interrupt handlers.
    // RTFM requires that free interrupts are declared in an 'extern block' when using software tasks;
    // some of these free interrupts will be used to dispatch the software tasks. An advantage of software
    // tasks over hardware tasks is that many tasks can be mapped to a single interrupt handler.
    extern "C" {
        fn DMA1_CH1(); // Not using the DMA1_CH1 interrupt
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
