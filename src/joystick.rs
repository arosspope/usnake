// use core::error::Error;
use nb;
use hal::{
    prelude::*,
    rcc::{Clocks, Rcc},
    adc::Adc,
    stm32::{Peripherals, GPIOB, GPIOA, ADC1, ADC2},
    gpio::{*, gpioa::*, gpiob::*, gpioc::*}
};
use core::f32::consts::PI;
use m::Float; // this trait provides the `atan2` method

#[derive(Debug, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}

pub struct Joystick {
    pub adc_x: Adc<ADC1>, // TODO: Investigate why two adc's are required to get accurate read of x / y
    pub adc_y: Adc<ADC2>,
    pub x: PA0<Analog>,
    pub y: PA4<Analog>,
    pub switch: PA2<Input<PullUp>>
}

#[derive(Debug)]
pub struct Error;

impl From<core::convert::Infallible> for Error {
    fn from(_: core::convert::Infallible) -> Self {
        Error {}
    }
}

impl From<nb::Error<()>> for Error {
    fn from(_: nb::Error<()>) -> Self {
        Error {}
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error {}
    }
}


impl Joystick {

    pub fn new() -> Self {
        // TODO: Generalise - Don't tie to a specific ADC / PIN mapping
        let mut dp      = hal::stm32::Peripherals::take().unwrap();
        let mut rcc     = dp.RCC.constrain();
        let mut flash   = dp.FLASH.constrain();
        let clocks      = rcc.cfgr.freeze(&mut flash.acr);
        let mut gpioa   = dp.GPIOA.split(&mut rcc.ahb);

        Joystick {
            adc_x: Adc::adc1(dp.ADC1, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
            adc_y: Adc::adc2(dp.ADC2, &mut dp.ADC1_2, &mut rcc.ahb, clocks),
            x: gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            y: gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            switch: gpioa.pa2.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr)
        }
    }


    /// Will return 'None' when the joystick is centered and not moving
    ///
    pub fn direction(&mut self) -> Result<Option<Direction>, Error> {
        let (x, y) = self.raw_xy()?;


        // const centre = 3200
        //
        // let direction = if y
        //
        //
        //
        //
        //
        //
        //
        //     Some(Direction::SouthEast)
        // } else if x < 3100 {
        //     None
        // };



        let theta = (-(y as i32 - 3200) as f32).atan2(-(x as i32 - 3200) as f32); // radians
        let direction = if theta < ((-7.0 * PI) / 8.0) {
            Some(Direction::North)
        } else if theta < ((-5.0 * PI) / 8.0) {
            Some(Direction::NorthWest)
        } else if theta < ((-3.0 * PI) / 8.0) {
            Some(Direction::West)
        } else if theta < ((-PI) / 8.0) {
            Some(Direction::SouthWest)
        } else if theta < ((PI) / 8.0) {
            Some(Direction::South)
        } else if theta < ((3.0 * PI) / 8.0) {
            Some(Direction::SouthEast)
        } else if theta < ((5.0 * PI) / 8.0) {
            Some(Direction::East)
        } else if theta < ((7.0 * PI) / 8.0) {
            Some(Direction::NorthEast)
        } else {
            None
        };


        // Ok(direction)
        Ok(direction)
    }

    pub fn raw_xy(&mut self) -> Result<(u16, u16), Error> {
        Ok((self.adc_x.read(&mut self.x)?, self.adc_y.read(&mut self.y)?))
    }

    pub fn is_pressed(&self) -> Result<bool, Error> {
        Ok(self.switch.is_low()?)
    }
}




//
//
//
// pub struct AdcJoystick<PINX, PINY> {
//     pub adc: Adc<ADC1>,
//     pub x: PINX,
//     pub y: PINY,
// }
//
// pub trait Joystick {
//     fn read(&mut self) -> Direction;
// }
//
// impl<PINX, PINY> Joystick for AdcJoystick<PINX, PINY>
// where
//     PINX: embedded_hal::adc::Channel<ADC1, ID = u8>,
//     PINY: embedded_hal::adc::Channel<ADC1, ID = u8>,
// {
//     fn read(&mut self) -> Direction {
//         let sample_x = self.adc.convert(&self.x, SampleTime::Cycles_480);
//         let x = self.adc.sample_to_millivolts(sample_x);
//
//         let sample_y = self.adc.convert(&self.y, SampleTime::Cycles_480);
//         let y = self.adc.sample_to_millivolts(sample_y);
//
//         if x < 1000 {
//             return Direction::Left;
//         }
//
//         if x > 2000 {
//             return Direction::Right;
//         }
//
//         if y < 1000 {
//             return Direction::Down;
//         }
//
//         if y > 2000 {
//             return Direction::Up;
//         }
//
//         Direction::Center
//     }
// }
