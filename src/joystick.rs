use core::{cmp, ops::Range};
use hal::{
    adc::Adc,
    gpio::{gpioa::*, *},
    prelude::*,
    stm32::{ADC1, ADC2},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn opposite(&self, other: &Direction) -> bool {
        match self {
            Direction::North => *other == Direction::South,
            Direction::South => *other == Direction::North,
            Direction::East => *other == Direction::West,
            Direction::West => *other == Direction::East,
            Direction::NorthEast => *other == Direction::SouthWest,
            Direction::SouthWest => *other == Direction::NorthEast,
            Direction::SouthEast => *other == Direction::NorthWest,
            Direction::NorthWest => *other == Direction::SouthEast,
        }
    }
}

pub struct Joystick {
    adc_x: Adc<ADC1>,
    adc_y: Adc<ADC2>,
    x: PA0<Analog>,
    y: PA4<Analog>,
    switch: PA2<Input<PullUp>>,
    dead_zone: Range<u16>,
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
    pub fn from_pins(
        adc_x: Adc<ADC1>,
        adc_y: Adc<ADC2>,
        x: PA0<Analog>,
        y: PA4<Analog>,
        switch: PA2<Input<PullUp>>,
    ) -> Result<Self, Error> {
        let mut joystick = Joystick {
            adc_x,
            adc_y,
            x,
            y,
            switch,
            dead_zone: 0..0,
        };

        // Find the deadzone of the joystick
        joystick.calibrate()?;

        Ok(joystick)
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

        x /= SAMPLE_SIZE;
        y /= SAMPLE_SIZE;

        let direction = if self.dead_zone.contains(&(x as u16)) && (y as u16) > self.dead_zone.end {
            Some(Direction::South)
        } else if self.dead_zone.contains(&(x as u16)) && (y as u16) < self.dead_zone.start {
            Some(Direction::North)
        } else if self.dead_zone.contains(&(y as u16)) && (x as u16) > self.dead_zone.end {
            Some(Direction::East)
        } else if self.dead_zone.contains(&(y as u16)) && (x as u16) < self.dead_zone.start {
            Some(Direction::West)
        } else if (x as u16) < self.dead_zone.start && (y as u16) < self.dead_zone.start {
            Some(Direction::NorthWest)
        } else if (x as u16) > self.dead_zone.end && (y as u16) < self.dead_zone.start {
            Some(Direction::NorthEast)
        } else if (x as u16) > self.dead_zone.end && (y as u16) > self.dead_zone.end {
            Some(Direction::SouthEast)
        } else if (x as u16) < self.dead_zone.start && (y as u16) > self.dead_zone.end {
            Some(Direction::SouthWest)
        } else {
            None
        };

        Ok(direction)
    }

    pub fn raw_xy(&mut self) -> Result<(u16, u16), Error> {
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

    fn calibrate(&mut self) -> Result<(), Error> {
        const SAMPLE_SIZE: u16 = 64;

        let initial_reading = self.raw_x()?;
        let mut max = initial_reading;
        let mut min = initial_reading;

        for _ in 0..SAMPLE_SIZE {
            let x_y = self.raw_xy()?;
            max = cmp::max(max, cmp::max(x_y.0, x_y.1));
            min = cmp::min(min, cmp::min(x_y.0, x_y.1));
        }

        // Increase the dead zone by about 10% on either side
        self.dead_zone = (min as f32 * 0.90) as u16..(max as f32 * 1.10) as u16;
        Ok(())
    }
}
