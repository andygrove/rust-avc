extern crate navigation;

use navigation::*;


struct GPS {
    filename: &'static str
}

impl GPS {

    fn new(f: &'static str) -> Self {
        GPS { filename: f }
    }

    fn get(&self) -> Location {
        //TODO: get real location
        Location::new(39.8617, -104.6731)
    }
}

fn main() {
    println!("Hello, world!");

    let gps = GPS::new("/dev/ttyUSB0");

    let test = gps.get();

    println!("Location: {:?}", test);
}
