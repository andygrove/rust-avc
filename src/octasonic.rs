extern crate spidev;
use self::spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

pub struct Octasonic {
    options: SpidevOptions,
    spi: Spidev,
}

impl Octasonic {
    pub fn new() -> Self {
        let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
        let mut options = SpidevOptions::new();

        options.bits_per_word(8)
            .max_speed_hz(20_000)
            .mode(SPI_MODE_0);

        spi.configure(&options).unwrap();

        Octasonic {
            spi: spi,
            options: options,
        }
    }

    pub fn set_sensor_count(&self, n: u8) {
        assert!(n > 0 && n < 9);
        let _ = self.transfer(0x10 | n);
    }

    pub fn get_sensor_count(&self) -> u8 {
        let _ = self.transfer(0x20);
        let n = self.transfer(0x00);
        // assert!(n>0 && n<9);
        n
    }

    pub fn get_sensor_reading(&self, n: u8) -> u8 {
        let _ = self.transfer(0x30 | n);
        self.transfer(0x00)
    }

    pub fn transfer(&self, b: u8) -> u8 {
        let mut transfer = SpidevTransfer::write(&[b]);
        self.spi.transfer(&mut transfer).unwrap();
        // println!("Sent: {:?}, Received: {:?}", b, transfer.rx_buf);
        let b = transfer.rx_buf.unwrap();
        (*b)[0]
    }
}
