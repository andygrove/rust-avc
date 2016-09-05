use std::io;
use std::io::prelude::*;

pub struct Octasonic {
}

impl Octasonic {

    pub fn new() -> Self {
        Octasonic {}
    }

    pub fn set_sensor_count(&self, n: u8) {
    }

    pub fn get_sensor_count(&self) -> u8 {
        0
    }

    pub fn get_sensor_reading(&self, n: u8) -> u8 {
        0
    }

}
