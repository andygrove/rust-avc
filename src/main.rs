
extern crate getopts;
extern crate chrono;

use getopts::Options;

use chrono::UTC;
use chrono::DateTime;

use std::env;
use std::thread;
use std::time::Duration;

mod navigation;
mod gps;
mod compass;
mod motors;
mod video;

use navigation::*;
use gps::GPS;
use compass::Compass;
use motors::Motors;
use video::Video;

#[derive(Debug)]
pub enum Action {
    Initializing,
    Navigating { waypoint: usize },
    ReachedWaypoint { waypoint: usize },
    WaitingForGps,
    AvoidingObstacleToLeft,
    AvoidingObstacleToRight,
    EmergencyStop,
    Finished
}

pub struct Car {
    gps: GPS,
    compass: Compass,
    motors: Motors,
    usonic: [u8; 5],
    action: Action
}

impl Car {

    fn set_action(&mut self, a: Action) {
        println!("Setting action to {:?}", a);
        self.action = a;
    }

}

fn close_enough(a: &Location, b: &Location) -> bool {
    (a.lat - b.lat).abs() < 0.000025 && (a.lon - b.lon).abs() < 0.000025
}

fn calc_bearing_diff(current_bearing: f64, wp_bearing: f64) -> f64 {
    let mut ret = wp_bearing - current_bearing;
    if ret < -180_f64 {
        ret += 360_f64;
    }
    else if ret > 180_f64 {
        ret -= 360_f64;
    }
    ret
}

fn navigate_to_waypoint(car: &mut Car, wp_num: usize, wp: &Location) {
    car.set_action(Action::Navigating { waypoint: wp_num });
    loop {
        match car.gps.get() {
            None => {
                car.set_action(Action::WaitingForGps);
                car.motors.stop();
            },
            Some(loc) => {
                if close_enough(&loc, &wp) {
                    car.set_action(Action::ReachedWaypoint { waypoint: wp_num });
                    break;
                }

                let current_bearing = car.compass.get();
                let wp_bearing = loc.calc_bearing_to(&wp);

                let turn_amount = calc_bearing_diff(current_bearing, wp_bearing);

                if turn_amount < 0_f64 {
                    car.motors.set_speed(100, 200);
                } else {
                    car.motors.set_speed(200, 100);
                }
            }
        };
        thread::sleep(Duration::from_millis(100));
    }
}

fn avc() {

    let mut car = Car {
        gps: GPS::new("/dev/ttyUSB0"),
        compass: Compass::new("/dev/ttyUSB1"),
        motors: Motors::new("/dev/ttyUSB2"),
        usonic: [0_u8; 5],
        action: Action::Initializing
    };

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
        navigate_to_waypoint(&mut car, i+1, &waypoint);
    }

    car.set_action(Action::Finished);
    car.motors.stop();

    println!("Finished");
}

fn test_gps() {
    println!("Testing GPS");
    let gps = GPS::new("/dev/ttyUSB0");
    gps.start_thread();
    loop {
        println!("GPS: {:?}", gps.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn test_video() {
    let video = Video::new(0);
    video.init().unwrap();

    let start = UTC::now().timestamp();

    for i in 0..100 {
        let now = UTC::now().timestamp();
        let elapsed = now - start;
        video.capture();
        if elapsed > 0 {
            video.drawText(30, 30, format!("Rendered {} frames in {} ms", i+1, elapsed));
            video.drawText(30, 50, format!("FPS: {}", (i+1) / elapsed));
        } else {
            video.drawText(30, 30, format!("Frame: {}", i));
        }
        video.write();
    }

    video.close();
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let _ = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "", "set video output file name", "out.mp4");
    opts.optflag("g", "test-gps", "tests the GPS");
    opts.optflag("v", "test-video", "tests the video");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if      matches.opt_present("g") { test_gps(); }
    else if matches.opt_present("v") { test_video(); }
    else {
        avc();
    }

}
