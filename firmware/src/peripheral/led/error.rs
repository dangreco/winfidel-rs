use esp_hal::ledc;

/// Errors that can occur when configuring or using the LED driver.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("esp_hal::ledc::channel::Error: {}", format_channel_error(.0))]
    Channel(ledc::channel::Error),
}

impl From<ledc::channel::Error> for Error {
    fn from(e: ledc::channel::Error) -> Self {
        Self::Channel(e)
    }
}

/// A convenient alias for `Result` types returned by this module.
pub type Result<T> = core::result::Result<T, Error>;

/// Converts a `ledc::channel::Error` into a human-readable string.
fn format_channel_error(e: &ledc::channel::Error) -> &'static str {
    match e {
        ledc::channel::Error::Channel => "channel not configured",
        ledc::channel::Error::Duty => "invalid duty % value",
        ledc::channel::Error::Timer => "timer not configured",
        ledc::channel::Error::Fade(err) => match err {
            ledc::channel::FadeError::Duration => {
                "duration too long for timer frequency and duty resolution"
            }
            ledc::channel::FadeError::DutyRange => {
                "duty % change from start to end is out of range"
            }
            ledc::channel::FadeError::EndDuty => "end duty % out of range",
            ledc::channel::FadeError::StartDuty => "start duty % out of range",
        },
    }
}
