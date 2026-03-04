#![no_std]

use embedded_hal_async::i2c::I2c;

/// Driver for the MCP3221 12-bit ADC.
pub struct Mcp3221<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Mcp3221<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Reads a 12-bit value from the ADC and returns it as a `u16`.
    pub async fn read(&mut self) -> Result<u16, E> {
        let mut buf = [0; 2];
        self.i2c.write_read(self.address, &[0], &mut buf).await?;
        Ok(u16::from_be_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::vec;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    const ADDR: u8 = 0x4D;

    #[tokio::test]
    async fn read_mid_range() {
        let expectations = [I2cTransaction::write_read(ADDR, vec![0], vec![0x07, 0xFF])];
        let i2c = I2cMock::new(&expectations);
        let mut adc = Mcp3221::new(i2c, ADDR);

        assert_eq!(adc.read().await.unwrap(), 0x07FF);
        adc.i2c.done();
    }

    #[tokio::test]
    async fn read_zero() {
        let expectations = [I2cTransaction::write_read(ADDR, vec![0], vec![0x00, 0x00])];
        let i2c = I2cMock::new(&expectations);
        let mut adc = Mcp3221::new(i2c, ADDR);

        assert_eq!(adc.read().await.unwrap(), 0);
        adc.i2c.done();
    }

    #[tokio::test]
    async fn read_max_12bit() {
        let expectations = [I2cTransaction::write_read(ADDR, vec![0], vec![0x0F, 0xFF])];
        let i2c = I2cMock::new(&expectations);
        let mut adc = Mcp3221::new(i2c, ADDR);

        assert_eq!(adc.read().await.unwrap(), 4095);
        adc.i2c.done();
    }

    #[tokio::test]
    async fn read_multiple_sequential() {
        let expectations = [
            I2cTransaction::write_read(ADDR, vec![0], vec![0x01, 0x00]),
            I2cTransaction::write_read(ADDR, vec![0], vec![0x02, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut adc = Mcp3221::new(i2c, ADDR);

        assert_eq!(adc.read().await.unwrap(), 0x0100);
        assert_eq!(adc.read().await.unwrap(), 0x0200);
        adc.i2c.done();
    }

    #[tokio::test]
    async fn read_custom_address() {
        let addr = 0x4E;
        let expectations = [I2cTransaction::write_read(addr, vec![0], vec![0x03, 0xAB])];
        let i2c = I2cMock::new(&expectations);
        let mut adc = Mcp3221::new(i2c, addr);

        assert_eq!(adc.read().await.unwrap(), 0x03AB);
        adc.i2c.done();
    }
}
