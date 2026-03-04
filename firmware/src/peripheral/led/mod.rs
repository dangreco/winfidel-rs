use driver_led::Led;
use esp_hal::{
    ledc::{self, LSGlobalClkSource, Ledc, LowSpeed, timer::TimerIFace},
    peripherals::Peripherals,
    time::Rate,
};
use static_cell::StaticCell;

use crate::peripheral;

mod error;
pub use error::*;

mod channel;
pub use channel::*;

static LEDC: StaticCell<Ledc> = StaticCell::new();
static LEDC_TIMER: StaticCell<ledc::timer::Timer<'static, LowSpeed>> = StaticCell::new();

/// Gets the onboard RGB LED driver, consuming the necessary peripherals.
pub fn onboard(
    peripherals: &'static mut Peripherals,
) -> Led<Channel<'static, LowSpeed>, Channel<'static, LowSpeed>, Channel<'static, LowSpeed>> {
    let ledc = LEDC.init(Ledc::new(unsafe { peripheral::take(&peripherals.LEDC) }));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let timer = LEDC_TIMER.init(ledc.timer::<LowSpeed>(ledc::timer::Number::Timer0));
    timer
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty5Bit,
            clock_source: ledc::timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(24),
        })
        .unwrap();

    let r = Channel::new(
        timer,
        ledc::channel::Number::Channel0,
        unsafe { peripheral::take(&peripherals.GPIO5) },
        true,
    );
    let g = Channel::new(
        timer,
        ledc::channel::Number::Channel1,
        unsafe { peripheral::take(&peripherals.GPIO4) },
        true,
    );
    let b = Channel::new(
        timer,
        ledc::channel::Number::Channel2,
        unsafe { peripheral::take(&peripherals.GPIO3) },
        true,
    );

    Led::new(r, g, b)
}
