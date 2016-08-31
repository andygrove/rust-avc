struct Octasonic {}

impl Octasonic {

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
        //TODO: SPI transfer
        0
    }

}