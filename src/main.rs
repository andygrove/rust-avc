
extern crate navigation;
extern crate getopts;

use navigation::*;

use getopts::Options;

use std::env;
use std::thread;
use std::time::Duration;

mod gps;
mod compass;
mod motors;
mod video;

use gps::GPS;
use compass::Compass;
use motors::Motors;
use video::Video;

pub enum Action {
    Initializing,
    Navigating { waypoint: Location },
    AvoidingObstacleToLeft,
    AvoidingObstacleToRight,
    Finished
}

pub struct Car {
    gps: GPS,
    compass: Compass,
    motors: Motors,
    usonic: [u8; 5],
    action: Action
}

fn close_enough(a: &Location, b: &Location) -> bool {
    (a.lat - b.lat).abs() < 0.000025 && (a.lon - b.lon).abs() < 0.000025
}

fn navigate_to_waypoint(car: &Car, wp: &Location) {
    loop {
        match car.gps.get() {
            None => {
                println!("Lost GPS signal");
                car.motors.stop();
            },
            Some(loc) => {
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
        };
        thread::sleep(Duration::from_millis(100));
    }
}

fn avc(car: &Car) {

    //    let video = Video::new(0);
    //    video.init();
    //TODO: Start video capture thread

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

fn test_gps(car: &Car) {
    println!("Testing GPS");
    car.gps.start_thread();
    loop {
        println!("GPS: {:?}", car.gps.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn main() {

    let car = Car {
        gps: GPS::new("/dev/ttyUSB0"),
        compass: Compass::new("/dev/ttyUSB1"),
        motors: Motors::new("/dev/ttyUSB2"),
        usonic: [0_u8; 5],
        action: Action::Initializing
    };

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "", "set video output file name", "out.mp4");
    opts.optflag("g", "test-gps", "tests the GPS");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("g") {
        test_gps(&car);
    } else {
        avc(&car);
    }

}
