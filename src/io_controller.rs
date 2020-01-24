use hal::gpio::{*, gpiob::*};
use max7219::{*, connectors::*};
use crate::joystick::Joystick;

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

impl From<max7219::PinError> for Error {
    fn from(_: max7219::PinError) -> Self {
        Error {}
    }
}


impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error {}
    }
}

pub type DisplayPins = PinConnector<PB8<Output<PushPull>>, PB9<Output<PushPull>>, PB10<Output<PushPull>>>;
pub struct IOController {
    pub joystick: Joystick,
    display: MAX7219<DisplayPins>,
    display_is_on: bool,

}

impl IOController {
    pub fn from(joystick: Joystick, display: MAX7219<DisplayPins>) -> Result<Self, Error> {
        let mut controller = IOController {
            joystick: joystick,
            display: display,
            display_is_on: true,
        };

        // Start the IOController in a known state
        controller.reset_display()?;
        controller.turn_off_display()?;
        Ok(controller)
    }

    pub fn write_display(&mut self, input: &[u8; 8]) -> Result<(), Error> {
        if !self.display_is_on {
            self.turn_on_display()?;
        }

        self.display.write_raw(0, &input)?;

        Ok(())
    }

    pub fn turn_off_display(&mut self) -> Result<(), Error> {
        if !self.display_is_on {
            return Err(Error);
        }

        self.display.power_off()?;
        self.display_is_on = false;
        Ok(())
    }

    pub fn turn_on_display(&mut self) -> Result<(), Error> {
        if self.display_is_on {
            return Err(Error);
        }

        self.display.power_on()?;
        self.display_is_on = true;
        Ok(())
    }

    pub fn toggle_display(&mut self) -> Result<(), Error> {
        if self.display_is_on {
            self.turn_off_display()?;
        } else {
            self.turn_on_display()?;
        }

        Ok(())
    }

    pub fn reset_display(&mut self) -> Result<(), Error> {
        Ok(self.display.clear_display(0)?)
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), Error> {
        if brightness > 100 {
            return Err(Error);
        }

        let brightness = (brightness as f32 * 2.55) as u8;
        self.display.set_intensity(0, brightness)?;
        Ok(())
    }
}
