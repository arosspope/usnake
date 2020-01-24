#![no_main]
#![no_std]
#![allow(unused_imports)]
use cortex_m::{self, iprintln, itm, peripheral::ITM};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    adc::*,
    prelude::*,
    time::{Hertz, MonoTimer},
};
use panic_itm;
use rtfm::{app, cyccnt::U32Ext, Exclusive};

use max7219::*;
use usnake::{game::*, io_controller::*, joystick::*};

#[app(device = hal::stm32, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const SNAKE: () = {
    // Late resources
    struct Resources {
        logger: ITM,
        game: Game,
        io_controller: IOController,
        sysclk_hz: Hertz,
    }

    #[init(spawn = [idle_screen])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core: rtfm::Peripherals         = cx.core; // Cortex-M peripherals
        let mut device: hal::stm32::Peripherals = cx.device; // Device specific peripherals

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
        let display     = MAX7219::from_pins(1, data, cs, sck).unwrap();

        // Setup Joystick
        let mut gpioa = device.GPIOA.split(&mut rcc.ahb);
        let joystick = Joystick::from_pins(
            Adc::adc1(device.ADC1, &mut device.ADC1_2, &mut rcc.ahb, clocks),
            Adc::adc2(device.ADC2, &mut device.ADC1_2, &mut rcc.ahb, clocks),
            gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr),
        )
        .unwrap();

        // Initialise IOController and Game objects
        let game                = Game::new(MonoTimer::new(core.DWT, clocks).now());
        let mut io_controller   = IOController::from(joystick, display).unwrap();

        iprintln!(&mut core.ITM.stim[0], "... app start ...");
        io_controller.set_brightness(100).unwrap();
        cx.spawn.idle_screen().unwrap();

        init::LateResources {
            logger: core.ITM,
            game: game,
            sysclk_hz: clocks.sysclk(),
            io_controller: io_controller,
        }
    }

    #[task(spawn = [game_tick], resources = [logger, game, io_controller, sysclk_hz], schedule = [idle_screen])]
    fn idle_screen(cx: idle_screen::Context) {
        // Update idlescreen
        static mut SCREEN: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        cx.resources.io_controller.write_display(SCREEN).ok();
        for i in 0..SCREEN.len() {
            SCREEN[i] = ((SCREEN[i] + 1) as u16 % 256) as u8 % 255;
        }

        // Check for user input - indicates that they are ready to play...
        if cx.resources.io_controller.joystick.is_pressed().unwrap() {
            logprintln!(Exclusive(cx.resources.logger), "... game start ...");
            cx.resources.game.reset();
            cx.spawn.game_tick().ok();
        } else {
            schedule_delayed_task!(cx, idle_screen, 150e-3);
        }
    }

    #[task(spawn = [game_over], resources = [logger, game, io_controller, sysclk_hz], schedule = [game_tick])]
    fn game_tick(cx: game_tick::Context) {
        let user_input = cx
            .resources
            .io_controller
            .joystick
            .direction()
            .expect("unable to read from joystick");

        // Tick the game and refresh the screen
        let state = cx.resources.game.tick(user_input);
        cx.resources
            .io_controller
            .write_display(&cx.resources.game.render())
            .ok();

        // Determine what to do next based on the state after the game tick
        match state {
            GameState::Running => {
                schedule_delayed_task!(cx, game_tick, 200e-3);
            }
            GameState::GameOver => {
                let score = cx.resources.game.score();
                logprintln!(Exclusive(cx.resources.logger), "... game end - final score: {} ...", score);
                cx.spawn.game_over().unwrap();
            }
        }
    }

    #[task(spawn = [idle_screen], resources = [io_controller, sysclk_hz], schedule = [game_over])]
    fn game_over(cx: game_over::Context) {
        static mut FLASH_SCREEN_COUNT: u8 = 0;

        if *FLASH_SCREEN_COUNT < 10 {
            *FLASH_SCREEN_COUNT += 1;
            cx.resources.io_controller.toggle_display().ok();
            schedule_delayed_task!(cx, game_over, 500e-3);
        } else {
            *FLASH_SCREEN_COUNT = 0;
            cx.spawn.idle_screen().unwrap();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    // Software tasks can also be assigned priorities and, under the hood, are dispatched from interrupt handlers.
    // RTFM requires that free interrupts are declared in an 'extern block' when using software tasks;
    // some of these free interrupts will be used to dispatch the software tasks.
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

/// Macros for sending a formatted string through ITM (provided by cortex_m)
#[macro_export]
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

#[macro_export]
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

// Macro for scheduling a task in rtfm
#[macro_export]
macro_rules! schedule_delayed_task {
    ($context:expr, $task:ident, $delay:expr) => {
        let delay = (($delay as f32 * $context.resources.sysclk_hz.0 as f32) as u32).cycles();
        $context.schedule.$task($context.scheduled + delay).unwrap();
    };
}
