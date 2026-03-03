pub mod error;
pub use error::*;

mod channel;
use channel::Channel;

use esp_hal::ledc;

/// A simple RGB LED driver.
pub struct Led<'d> {
    r: Channel<'d, ledc::LowSpeed>,
    g: Channel<'d, ledc::LowSpeed>,
    b: Channel<'d, ledc::LowSpeed>,
    power: bool,
    state: (u8, u8, u8),
}

impl<'d> Led<'d> {
    pub fn new(
        timer: &'d ledc::timer::Timer<'d, ledc::LowSpeed>,
        r: impl esp_hal::gpio::OutputPin + 'd,
        g: impl esp_hal::gpio::OutputPin + 'd,
        b: impl esp_hal::gpio::OutputPin + 'd,
    ) -> Self {
        let r = Channel::new(timer, ledc::channel::Number::Channel0, r, true);
        let g = Channel::new(timer, ledc::channel::Number::Channel1, g, true);
        let b = Channel::new(timer, ledc::channel::Number::Channel2, b, true);

        Self {
            r,
            g,
            b,
            power: false,
            state: (0, 0, 0),
        }
    }

    #[inline]
    fn set(&mut self, state: (u8, u8, u8)) -> Result<()> {
        self.r.set(state.0)?;
        self.g.set(state.1)?;
        self.b.set(state.2)?;
        Ok(())
    }

    /// Turn on the LED.
    pub fn on(&mut self) -> Result<()> {
        self.power = true;
        self.set(self.state)
    }

    /// Turn off the LED.
    pub fn off(&mut self) -> Result<()> {
        self.power = false;
        self.set((0, 0, 0))
    }

    /// Toggle the LED.
    pub fn toggle(&mut self) -> Result<()> {
        if self.power { self.off() } else { self.on() }
    }

    /// Set the color of the LED.
    pub fn color(&mut self, r: u8, g: u8, b: u8) -> Result<()> {
        self.state = (r, g, b);
        if self.power {
            self.set(self.state)
        } else {
            Ok(())
        }
    }
}
