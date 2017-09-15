extern crate hmc5883l;
use self::hmc5883l::*;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use std::f32::consts::PI;
use std::f32;

use std::io::prelude::*;

pub struct Compass {
    mag: HMC5883L
}

impl Compass {

    pub fn new(f: &'static str) -> Self {
        Compass {
            mag: HMC5883L::new(f, 0x1E).unwrap()
        }
    }

    pub fn get(&mut self) -> Option<f32> {

        let gauss_lsb_xy = 1100.0;
        let gauss_lsb_z  =  980.0;

        // You need to determine the correct magnetic declination for your location for accurate
        // readings. Find yours at http://www.magnetic-declination.com/
        let declination_angle = 0.22; // in radians, not degrees

         // read raw values
        let (x, y, z) = self.mag.read().unwrap();

        // convert to micro-teslas
        let (x, y, _) = (x/gauss_lsb_xy*100.0, y/gauss_lsb_xy*100.0, z/gauss_lsb_z*100.0);

        let mut heading = y.atan2(x) + declination_angle;

        if heading < 0.0 {
            heading += 2.0 * PI;
        }

        if heading > 2.0 * PI {
            heading -= 2.0 * PI;
        }

        // Convert radians to degrees for readability.
        heading = heading * 180.0 / PI;

        Some(heading)
    }
}
