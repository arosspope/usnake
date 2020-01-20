use nb;
use hal::{
    prelude::*,
    rcc::{Clocks, Rcc},
    adc::Adc,
    stm32::{Peripherals, GPIOB, GPIOA, ADC1, ADC2},
    gpio::{*, gpioa::*, gpiob::*, gpioc::*}
};

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
        const SAMPLE_SIZE: u32 = 16;
        // Average the x / y over a sample set
        let mut x: u32 = 0;
        let mut y: u32 = 0;
        for _ in 0..SAMPLE_SIZE {
            let x_y = self.raw_xy()?;
            x += x_y.0 as u32;
            y += x_y.1 as u32;
        }

        x = x / SAMPLE_SIZE;
        y = y / SAMPLE_SIZE;

        const CENTRE: core::ops::Range<u32> = 3150..3300;
        const RANGE_A: core::ops::Range<u32> = 3300..4100;
        const RANGE_B: core::ops::Range<u32> = 0..3150;


        // Would use atan2 here but the opposite directions on each axis do not appear
        // equidistant in their ADC readings
        // let theta = (y as f32).atan2(x as f32); // radians
        let direction = if CENTRE.contains(&x) && RANGE_A.contains(&y) {
            Some(Direction::South)
        } else if CENTRE.contains(&x) && RANGE_B.contains(&y) {
            Some(Direction::North)
        } else if CENTRE.contains(&y) && RANGE_A.contains(&x) {
            Some(Direction::East)
        } else if CENTRE.contains(&y) && RANGE_B.contains(&x) {
            Some(Direction::West)
        } else if RANGE_B.contains(&x) && RANGE_B.contains(&y) {
            Some(Direction::NorthWest)
        } else if RANGE_A.contains(&x) && RANGE_B.contains(&y) {
            Some(Direction::NorthEast)
        } else if RANGE_A.contains(&x) && RANGE_A.contains(&y) {
            Some(Direction::SouthEast)
        } else if RANGE_B.contains(&x) && RANGE_A.contains(&y) {
            Some(Direction::SouthWest)
        } else {
            None
        };

        Ok(direction)
    }

    pub fn raw_xy(&mut self) -> Result<(u16, u16), Error> {
        // TODO: Probably need better precision in these readings...
        Ok((self.raw_x()?, self.raw_y()?))
    }

    pub fn raw_x(&mut self) -> Result<u16, Error> {
        Ok(self.adc_x.read(&mut self.x)?)
    }

    pub fn raw_y(&mut self) -> Result<u16, Error> {
        Ok(self.adc_y.read(&mut self.y)?)
    }

    pub fn is_pressed(&self) -> Result<bool, Error> {
        Ok(self.switch.is_low()?)
    }
}
