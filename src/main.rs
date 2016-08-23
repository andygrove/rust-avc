extern crate navigation;

use navigation::*;

mod gps;
mod compass;
mod motors;

use gps::GPS;
use compass::Compass;
use motors::Motors;

struct Car {
    gps: GPS,
    compass: Compass,
    motors: Motors,
    usonic: [u8; 5]
}

//enum Action {
//
//}


fn close_enough(a: &Location, b: &Location) -> bool {
    (a.lat - b.lat).abs() < 0.0001 && (a.lon - b.lon).abs() < 0.0001
}

fn navigate_to_waypoint(car: &Car, wp: &Location) {
    loop {
        let loc = car.gps.get().unwrap();
        println!("Current location: {:?}", loc);

        if close_enough(&loc, &wp) {
            println!("Reached waypoint!");
            break;
        }

        let actual_bearing = car.compass.get();
        let desired_bearing = loc.calc_bearing_to(&wp);

        //TODO: determine what speed to set the motors to
        car.motors.set_speed(100, 100);

    }
}


fn main() {

    let car = Car {
        gps: GPS::new("/dev/ttyUSB0"),
        compass: Compass::new("/dev/ttyUSB1"),
        motors: Motors::new("/dev/ttyUSB2"),
        usonic: [0_u8; 5]
    };

    //TODO: wait for start button

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
    ];

    // navigate to each waypoint in turn
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);
        navigate_to_waypoint(&car, &waypoint);
    }

    car.motors.stop();

    println!("Finished");

}
