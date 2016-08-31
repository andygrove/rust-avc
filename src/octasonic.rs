extern crate spidev;
use std::io;
use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

struct Octasonic {
    spi: Spidev
}

impl Octasonic {

    fn new() -> Self {
        let mut spi = try!(Spidev::open("/dev/spidev0.0"));
        let mut options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(20_000)
            .mode(SPI_MODE_0);
        try!(spi.configure(&options));

        Octasonic { spi: spi }
    }

    fn set_sensor_count(&self, n: u8) {
        assert!(n>0 && n<9);
        let _ = self.transfer(0x10 | n);
    }

    fn get_sensor_count(&self) -> u8 {
        let _ = self.transfer(0x20);
        let n = self.transfer(0x00);
        assert!(n>0 && n<9);
        n
    }

    fn get_sensor_reading(&self, n: u8) -> u8 {
        assert!(n>0 && n<9);
        let _ = self.transfer(0x30 | n);
        self.transfer(0x00)
    }

    fn transfer(&self, b: u8) -> u8 {
        let mut transfer = SpidevTransfer::write(&[b]);
        try!(spi.transfer(&mut transfer));
        println!("{:?}", transfer.rx_buf);
        transfer.rx_buf[0]
    }

}