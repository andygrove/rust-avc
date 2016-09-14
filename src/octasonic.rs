extern crate spidev;
use self::spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

pub struct Octasonic {
    spi: Spidev,
    /// the number of readings to average out, or 1 for no averaging
    avg: u8,
    values: Vec<[u8]>
}

impl Octasonic {

    pub fn new(sensor_count: u8, avg: u8) -> Result<(), spidev::Error> {
        assert!(sensor_count > 0 && sensor_count < 9);

        let mut spi = try!(Spidev::open("/dev/spidev0.0"));
        let mut options = try!(SpidevOptions::new());

        options.bits_per_word(8)
            .max_speed_hz(20_000)
            .mode(SPI_MODE_0);

        try!(spi.configure(&options));

        // set the sensor count
        let _ = self.transfer(0x10 | n);

        // initialize the result vector
        let v : Vec<[u8]> = Vec::with_capacity(sensor_count);
        for _ in 0..sensor_count {
            v.push(vec![0_u8; sensor_count]);
        }

        Ok(Octasonic {
            spi: spi,
            avg: avg,
            values: v
        })
    }

    pub fn read(&mut self, n: u8) -> u8 {
        let v = self.get_sensor_reading(n);
        if self.values[n].len() == self.avg {
            self.values[n].remove(0);
        }
        self.values[n].push(v);
        let t = 0;
        for i in 0..self.values[n].len() {
            t += self.values[n][i];
        }
        t / self.values[n].len()
    }

    pub fn get_sensor_count(&self) -> u8 {
        let _ = self.transfer(0x20);
        let n = self.transfer(0x00);
        n
    }

    /// get the raw value from the sensor without any averaging
    pub fn get_sensor_reading(&self, n: u8) -> u8 {
        let _ = self.transfer(0x30 | n);
        self.transfer(0x00)
    }

    fn transfer(&self, b: u8) -> u8 {
        let mut transfer = SpidevTransfer::write(&[b]);
        self.spi.transfer(&mut transfer).unwrap();
        let b = transfer.rx_buf.unwrap();
        (*b)[0]
    }
}
