
extern crate getopts;
extern crate chrono;
extern crate navigation;

use getopts::Options;

use chrono::UTC;
//use chrono::DateTime;

use std::env;
use std::thread;
use std::time::Duration;

mod gps;
mod compass;
mod motors;
mod video;
mod qik;

use navigation::*;
use gps::GPS;
use compass::Compass;
use motors::Motors;
use video::Video;
use qik::*;

#[derive(Debug)]
pub enum Action {
    Initializing,
    Navigating { waypoint: usize },
    ReachedWaypoint { waypoint: usize },
    WaitingForGps,
    WaitingForCompass,
    AvoidingObstacleToLeft,
    AvoidingObstacleToRight,
    EmergencyStop,
    Finished
}

pub struct Car {
    gps: GPS,
    compass: Compass,
    motors: Motors,
//    usonic: [u8; 5],
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

fn navigate_to_waypoint(video: &Video, enable_motors: bool, car: &mut Car, wp_num: usize, wp: &Location) {
    car.set_action(Action::Navigating { waypoint: wp_num });
    loop {

        video.capture();

        video.draw_text(30, 30, format!("UTC: {}", UTC::now()));

        match car.gps.get() {
            None => {
                video.draw_text(30, 30, format!("GPS: N/A"));
                car.set_action(Action::WaitingForGps);
                if enable_motors {
                    car.motors.coast();
                }
            },
            Some(loc) => {
                video.draw_text(30, 30, format!("GPS: {}, {}", loc.lat, loc.lon));
                if close_enough(&loc, &wp) {
                    car.set_action(Action::ReachedWaypoint { waypoint: wp_num });
                    break;
                }

                match car.compass.get() {
                    None => {
                        video.draw_text(30, 50, format!("Compass: N/A"));
                        car.set_action(Action::WaitingForCompass);
                        if enable_motors {
                            car.motors.coast();
                        }
                    },
                    Some(b) => {
                        video.draw_text(30, 50, format!("Compass: {}", b));
                        let wp_bearing = loc.calc_bearing_to(&wp);

                        let turn_amount = calc_bearing_diff(b, wp_bearing);

                        if enable_motors {
                            if turn_amount < 0_f64 {
                                car.motors.set_speed(100, 200);
                            } else {
                                car.motors.set_speed(200, 100);
                            }
                        }
                    }
                }
            }
        };

        video.draw_text(30, 100, format!("{:?}", car.action));
        video.write();
    }
}

fn avc(enable_motors: bool) {

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
    ];

    let mut car = Car {
        gps: GPS::new("/dev/gps"),
        compass: Compass::new("/dev/imu"),
        motors: Motors::new("/dev/qik"),
        //usonic: [0_u8; 5],
        action: Action::Initializing
    };

    let video = Video::new(0);
    let start = UTC::now().timestamp();

    video.init(format!("avc-{}.mp4", start)).unwrap();

    //TODO: start thread to do video capture

    //TODO: wait for start button

    // navigate to each waypoint in turn
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);
        navigate_to_waypoint(&video, enable_motors, &mut car, i+1, &waypoint);
    }

    car.set_action(Action::Finished);

    if enable_motors {
        car.motors.stop();
    }

    video.close();

    println!("Finished");
}

fn test_gps() {
    println!("Testing GPS");
    let gps = GPS::new("/dev/gps");
    gps.start_thread();
    loop {
        println!("GPS: {:?}", gps.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn test_compass() {
    println!("Testing Compass");
    let compass = Compass::new("/dev/imu");
    compass.start_thread();
    loop {
        println!("Compass: {:?}", compass.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn test_motors() {
    use qik::ConfigParam::*;
    let mut qik = qik::Qik::new(String::from("/dev/qik"), 123);
    qik.init();
    println!("Firmware version: {}", qik.get_firmware_version());
    println!("MOTOR_M0_ACCELERATION : {}", qik.get_config(MOTOR_M0_ACCELERATION));
    println!("MOTOR_M1_ACCELERATION : {}", qik.get_config(MOTOR_M1_ACCELERATION));
    for i in 0 .. 127 {
        qik.set_speed(Motor::M0, i);
        qik.set_speed(Motor::M1, i);
        std::thread::sleep_ms(30);
    }
    qik.set_speed(Motor::M0, 0);
    qik.set_speed(Motor::M1, 0);
}

fn test_video() {

    let video = Video::new(0);

    let start = UTC::now().timestamp();

    video.init(format!("video-test-{}.mp4", start)).unwrap();

    for i in 0..100 {
        let now = UTC::now().timestamp();
        let elapsed = now - start;
        video.capture();
        if elapsed > 0 {
            video.draw_text(30, 30, format!("Rendered {} frames in {} seconds", i+1, elapsed));
            video.draw_text(30, 50, format!("FPS: {:.*}", 1, (i+1) / elapsed));
        } else {
            video.draw_text(30, 30, format!("Frame: {}", i));
        }
        video.write();
    }

    video.close();
}

fn main() {

    println!("G-Force!");

    let args: Vec<String> = env::args().collect();
    let _ = args[0].clone();

    let mut opts = Options::new();
    opts.optopt( "o", "out", "set video output file name", "out.mp4");
    opts.optflag("g", "test-gps", "tests the GPS");
    opts.optflag("v", "test-video", "tests the video");
    opts.optflag("c", "test-compass", "tests the compass");
    opts.optflag("m", "test-motors", "tests the motors");
    opts.optflag("u", "test-ultrasonic", "tests the ultrasonic sensors");
    opts.optflag("t", "test-avc", "tests the full avc logic, but without motors running");
    opts.optflag("a", "avc", "navigate a course, for real!");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if      matches.opt_present("g") { test_gps(); }
    else if matches.opt_present("v") { test_video(); }
    else if matches.opt_present("c") { test_compass(); }
    else if matches.opt_present("m") { test_motors(); }
    else if matches.opt_present("u") { panic!("not implemented"); }
    else if matches.opt_present("t") { avc(false); }
    else if matches.opt_present("a") { avc(true); }
    else { panic!("missing cmd line argument .. try --help"); }

}
