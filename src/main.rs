extern crate navigation;

use navigation::*;

mod gps;
mod compass;

use gps::GPS;
use compass::Compass;

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
