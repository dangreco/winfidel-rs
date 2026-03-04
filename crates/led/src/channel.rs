/// Anything that can output a duty cycle (0-255)
pub trait PwmChannel {
    type Error;
    fn set_duty(&mut self, duty: u8) -> Result<(), Self::Error>;
}
