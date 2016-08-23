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

struct Compass {
    filename: &'static str
}

impl Compass {

    fn new(f: &'static str) -> Self {
        Compass { filename: f }
    }

    fn get(&self) -> f32 {
        //TODO: get real compass bearing
        349.5
    }

}

struct Motor {
    filename: &'static str
}

fn main() {
    println!("Hello, world!");

    let gps = GPS::new("/dev/ttyUSB0");
    let compass = Compass::new("/dev/ttyUSB1");

    let test = gps.get();


    println!("Location: {:?}", test);
}
