#![no_std]

mod channel;
pub use channel::*;

/// A simple RGB LED driver.
pub struct Led<R, G, B>
where
    R: PwmChannel,
    G: PwmChannel<Error = R::Error>,
    B: PwmChannel<Error = R::Error>,
{
    r: R,
    g: G,
    b: B,
    power: bool,
    state: (u8, u8, u8),
}

impl<R, G, B> Led<R, G, B>
where
    R: PwmChannel,
    G: PwmChannel<Error = R::Error>,
    B: PwmChannel<Error = R::Error>,
{
    pub fn new(r: R, g: G, b: B) -> Self {
        Self {
            r,
            g,
            b,
            power: false,
            state: (0, 0, 0),
        }
    }

    /// Set the LED to the specified RGB color.
    #[inline]
    fn set(&mut self, (r, g, b): (u8, u8, u8)) -> Result<(), R::Error> {
        self.r.set_duty(r)?;
        self.g.set_duty(g)?;
        self.b.set_duty(b)?;
        Ok(())
    }

    /// Turn on the LED.
    pub fn on(&mut self) -> Result<(), R::Error> {
        self.power = true;
        self.set(self.state)
    }

    /// Turn off the LED.
    pub fn off(&mut self) -> Result<(), R::Error> {
        self.power = false;
        self.set((0, 0, 0))
    }

    /// Toggle the LED.
    pub fn toggle(&mut self) -> Result<(), R::Error> {
        if self.power { self.off() } else { self.on() }
    }

    /// Set the color of the LED.
    pub fn color(&mut self, r: u8, g: u8, b: u8) -> Result<(), R::Error> {
        self.state = (r, g, b);
        if self.power {
            self.set(self.state)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::vec::Vec;

    struct MockChannel {
        duties: Vec<u8>,
    }

    impl MockChannel {
        fn new() -> Self {
            Self { duties: Vec::new() }
        }
        fn last(&self) -> u8 {
            *self.duties.last().unwrap_or(&0)
        }
    }

    impl PwmChannel for MockChannel {
        type Error = core::convert::Infallible;
        fn set_duty(&mut self, duty: u8) -> Result<(), Self::Error> {
            self.duties.push(duty);
            Ok(())
        }
    }

    fn make_led() -> Led<MockChannel, MockChannel, MockChannel> {
        Led::new(MockChannel::new(), MockChannel::new(), MockChannel::new())
    }

    #[test]
    fn starts_off_with_black() {
        let led = make_led();
        assert!(!led.power);
        assert_eq!(led.state, (0, 0, 0));
    }

    #[test]
    fn on_applies_current_state() {
        let mut led = make_led();
        led.color(255, 128, 64).unwrap();
        led.on().unwrap();
        assert!(led.power);
        assert_eq!(led.r.last(), 255);
        assert_eq!(led.g.last(), 128);
        assert_eq!(led.b.last(), 64);
    }

    #[test]
    fn off_zeros_channels() {
        let mut led = make_led();
        led.color(100, 100, 100).unwrap();
        led.on().unwrap();
        led.off().unwrap();
        assert!(!led.power);
        assert_eq!(led.r.last(), 0);
        assert_eq!(led.g.last(), 0);
        assert_eq!(led.b.last(), 0);
    }

    #[test]
    fn off_preserves_state() {
        let mut led = make_led();
        led.color(100, 100, 100).unwrap();
        led.on().unwrap();
        led.off().unwrap();
        assert_eq!(led.state, (100, 100, 100));
    }

    #[test]
    fn color_while_off_stores_but_doesnt_write() {
        let mut led = make_led();
        led.color(200, 100, 50).unwrap();
        assert_eq!(led.state, (200, 100, 50));
        assert!(led.r.duties.is_empty());
        assert!(led.g.duties.is_empty());
        assert!(led.b.duties.is_empty());
    }

    #[test]
    fn color_while_on_writes_immediately() {
        let mut led = make_led();
        led.on().unwrap();
        led.color(10, 20, 30).unwrap();
        assert_eq!(led.r.last(), 10);
        assert_eq!(led.g.last(), 20);
        assert_eq!(led.b.last(), 30);
    }

    #[test]
    fn toggle_off_to_on() {
        let mut led = make_led();
        led.color(50, 60, 70).unwrap();
        led.toggle().unwrap();
        assert!(led.power);
        assert_eq!(led.r.last(), 50);
        assert_eq!(led.g.last(), 60);
        assert_eq!(led.b.last(), 70);
    }

    #[test]
    fn toggle_on_to_off() {
        let mut led = make_led();
        led.on().unwrap();
        led.toggle().unwrap();
        assert!(!led.power);
        assert_eq!(led.r.last(), 0);
    }

    #[test]
    fn double_toggle_restores() {
        let mut led = make_led();
        led.color(42, 43, 44).unwrap();
        led.toggle().unwrap();
        led.toggle().unwrap();
        assert!(!led.power);
        assert_eq!(led.state, (42, 43, 44));
    }

    #[test]
    fn on_with_default_black() {
        let mut led = make_led();
        led.on().unwrap();
        assert_eq!(led.r.last(), 0);
        assert_eq!(led.g.last(), 0);
        assert_eq!(led.b.last(), 0);
    }
}
