extern crate navigation;

use navigation::*;

mod gps;
mod compass;
mod motors;

use gps::GPS;
use compass::Compass;
use motors::Motors;


//enum Action {
//
//}


fn main() {
    println!("Hello, world!");

    let gps = GPS::new("/dev/ttyUSB0");
    let compass = Compass::new("/dev/ttyUSB1");
    let motors = Motors::new("/dev/ttyUSB2");

    //TODO: wait for start button

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
    ];

    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);

        let test = gps.get().unwrap();

        println!("Location: {:?}", test);


    }

    println!("Finished");

}
