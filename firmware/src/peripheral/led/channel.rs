use driver_led::PwmChannel;
use esp_hal::ledc;

use esp_hal::gpio::DriveMode;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::ledc::channel::ChannelIFace;

use crate::peripheral::led::error::Result;

/// Maps a duty cycle percentage to a value between min and max, optionally inverting it.
#[inline(always)]
fn map(inverted: bool, min: u8, max: u8, value: u8) -> u8 {
    let clamped = value.clamp(min, max);
    if inverted { max - clamped } else { clamped }
}

/// A wrapper around `ledc::channel::Channel` that supports duty cycle inversion.
pub struct Channel<'d, S: ledc::timer::TimerSpeed> {
    internal: ledc::channel::Channel<'d, S>,
    inverted: bool,
}

impl<'d, S: ledc::timer::TimerSpeed> Channel<'d, S> {
    pub fn new(
        timer: &'d dyn ledc::timer::TimerIFace<S>,
        number: ledc::channel::Number,
        output_pin: impl PeripheralOutput<'d>,
        inverted: bool,
    ) -> Self {
        let mut channel = ledc::channel::Channel::new(number, output_pin);
        channel
            .configure(ledc::channel::config::Config {
                timer,
                drive_mode: DriveMode::PushPull,
                duty_pct: map(inverted, 0, 100, 0), // Off by default
            })
            .unwrap();

        Self {
            internal: channel,
            inverted,
        }
    }

    /// Sets the duty cycle percentage for this channel, applying inversion if configured.
    pub fn set(&mut self, duty: u8) -> Result<()> {
        self.internal
            .set_duty(map(self.inverted, 0, 100, duty))
            .map_err(Into::into)
    }
}

impl<'d, S: ledc::timer::TimerSpeed> PwmChannel for Channel<'d, S> {
    type Error = crate::peripheral::led::error::Error;

    fn set_duty(&mut self, duty: u8) -> Result<()> {
        self.set(duty)
    }
}
