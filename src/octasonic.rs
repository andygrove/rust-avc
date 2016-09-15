extern crate spidev;
use self::spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

use std::io::Error;

pub struct Octasonic {
    spi: Spidev,
    values: Vec<Vec<u8>>
}

impl Octasonic {

    /// Create an Octasonic struct for the specified sensor count
    pub fn new(sensor_count: usize, sample_count: usize) -> Result<Self, Error> {
        let mut spi = try!(Spidev::open("/dev/spidev0.0"));
        let mut options = SpidevOptions::new();

        options.bits_per_word(8)
            .max_speed_hz(20_000)
            .mode(SPI_MODE_0);

        try!(spi.configure(&options));

        // initialize history for each sensor with readings of 200 cm
        let mut v = Vec::with_capacity(sensor_count);
        for _ in 0..sensor_count {
            v.push(vec![ 200_u8; sensor_count]);
        }

        let o = Octasonic {
            spi: spi,
            values: v
        };

        o.set_sensor_count(sample_count as u8);

        Ok(o)
    }

    /// get the averaged sensor reading
    pub fn get_sensor_reading(&mut self, n: u8) -> u8 {
        // get the new reading
        let x = self._get_sensor_reading(n);
        // push the reading onto the history
        let mut v = &mut self.values[n as usize];
        v.remove(0);
        v.push(x);
        // calculate the average
        let mut total = 0_u32;
        for i in 0..v.len() {
            total += v[i] as u32;
        }
        (total as u32 / v.len() as u32) as u8
    }

    pub fn get_sensor_count(&self) -> u8 {
        let _ = self.transfer(0x20);
        self.transfer(0x00)
    }

    /// get the raw sensor reading
    fn _get_sensor_reading(&self, n: u8) -> u8 {
        let _ = self.transfer(0x30 | n);
        self.transfer(0x00)
    }

    pub fn set_sensor_count(&self, n: u8) {
        assert!(n > 0 && n < 9);
        let _ = self.transfer(0x10 | n);
    }

    fn transfer(&self, b: u8) -> u8 {
        let mut transfer = SpidevTransfer::write(&[b]);
        self.spi.transfer(&mut transfer).unwrap();
        // println!("Sent: {:?}, Received: {:?}", b, transfer.rx_buf);
        let b = transfer.rx_buf.unwrap();
        (*b)[0]
    }
}
