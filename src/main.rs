extern crate iron;
extern crate urlencoded;
extern crate getopts;
extern crate chrono;
extern crate navigation;
extern crate qik;

use getopts::Options;
use chrono::UTC;
use navigation::*;
use qik::*;
use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedQuery;

use std::collections::HashMap;
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod gps;
mod compass;
mod video;
mod avc;

use gps::GPS;
use compass::Compass;
use video::Video;
use avc::*;

pub struct Config {
    gps_device: &'static str,
    imu_device: &'static str,
    qik_device: &'static str
}

fn main() {

    println!("Welcome to G-Force!");

    let args: Vec<String> = env::args().collect();
    let _ = args[0].clone();

    let mut opts = Options::new();
    opts.optopt( "o", "out", "set video output file name", "out.mp4");
    opts.optflag("g", "test-gps", "tests the GPS");
    opts.optflag("v", "test-video", "tests the video");
    opts.optflag("i", "test-imu", "tests the IMU");
    opts.optflag("m", "test-motors", "tests the motors");
    opts.optflag("u", "test-ultrasonic", "tests the ultrasonic sensors");
    opts.optflag("t", "test-avc", "tests the full avc logic, but without motors running");
    opts.optflag("a", "avc", "Start the web server");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let conf = Config {
        gps_device: "/dev/gps",
        imu_device: "/dev/imu",
        qik_device: "/dev/qik",
    };

    if      matches.opt_present("g") { test_gps(&conf); }
    else if matches.opt_present("v") { test_video(&conf); }
    else if matches.opt_present("i") { test_imu(&conf); }
    else if matches.opt_present("m") { test_motors(&conf); }
    else if matches.opt_present("u") { panic!("not implemented"); }
    else if matches.opt_present("t") { run_avc(conf); }
    else if matches.opt_present("a") { run_avc(conf); }
    else { panic!("missing cmd line argument .. try --help"); }

}

fn run_avc(conf: Config) {

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.94177796143009, -105.08160397410393),
        Location::new(39.94190648894769, -105.08158653974533),
        Location::new(39.94186741660787, -105.08174613118172),
    ];

    //TODO: load settings from file
    let settings = Settings {
        max_speed: 127,
        differential_drive_coefficient: 5_f32,
        enable_motors: false,
        waypoints: waypoints
    };

    let avc = AVC::new(conf, settings);

    let start_state = avc.get_shared_state();
    let web_state = avc.get_shared_state();

    let avc_thread = thread::spawn(move || {

        // wait for the user to hit the start button
        println!("Waiting for start command");
        loop {
            {
                let mut state = start_state.lock().unwrap();
                match &state.action {
                    &   Action::WaitingForStartCommand => {},
                    _ => break
                };
            }
            thread::sleep(Duration::from_millis(10));
        }

        // run!
        avc.run();

    });

    Iron::new(move |req: &mut Request| {

        println!("URL: {}", req.url);

        match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref hashmap) => {
                match hashmap.get("action") {
                    Some(ref a) => {
                        match a[0].as_ref() {
                            "start" => {
                                {
                                    let mut state = web_state.lock().unwrap();
                                    state.set_action(Action::Navigating { waypoint: 1 });
                                }
                                Ok(Response::with((status::Ok, "<html><body><form action=\"stop\"><input type=\"submit\">Stop!</input></form></body></html>")))
                            },
                            "stop" => {
                                {
                                    let mut state = web_state.lock().unwrap();
                                    state.set_action(Action::Aborted);
                                }
                                Ok(Response::with((status::Ok, "Stopped! You'll need to restart the app now")))
                            },
                            _ => Ok(Response::with((status::Ok, "Did not recognize action")))
                        }
                    },
                    None => Ok(Response::with((status::Ok,
                                   "<html><body><form action=\"start\"><input type=\"submit\">Start!</input></form></body></html>")))
                }

            },
            Err(ref e) => {
                println!("{:?}", e);
                Ok(Response::with((status::Ok, "Error")))
            }
        }

    }).http("0.0.0.0:8080").unwrap();

}

fn test_gps(conf: &Config) {
    println!("Testing GPS");
    let gps = GPS::new(conf.gps_device);
    gps.start_thread();
    loop {
        println!("GPS: {:?}", gps.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn test_imu(conf: &Config) {
    println!("Testing IMU");
    let compass = Compass::new(conf.imu_device);
    compass.start_thread();
    loop {
        println!("Compass: {:?}", compass.get());
        thread::sleep(Duration::from_millis(1000));
    }
}

fn test_motors(conf: &Config) {
    println!("Testing motors");
    use qik::ConfigParam::*;
    let mut qik = qik::Qik::new(String::from(conf.qik_device), 123);
    qik.init();
    println!("Firmware version: {}", qik.get_firmware_version());
    println!("MOTOR_M0_ACCELERATION : {}", qik.get_config(MOTOR_M0_ACCELERATION));
    println!("MOTOR_M1_ACCELERATION : {}", qik.get_config(MOTOR_M1_ACCELERATION));
    for i in 0 .. 127 {
        qik.set_speed(Motor::M0, i);
        qik.set_speed(Motor::M1, i);
        std::thread::sleep(Duration::from_millis(30));
    }
    qik.set_speed(Motor::M0, 0);
    qik.set_speed(Motor::M1, 0);
}

fn test_video(conf: &Config) {

    let gps = GPS::new(conf.gps_device);
    gps.start_thread();

    let compass = Compass::new(conf.imu_device);
    compass.start_thread();

    let video = Video::new(0);

    let start = UTC::now().timestamp();

    video.init(format!("video-test-{}.mp4", start)).unwrap();

    let mut i = 0;
    loop {
        i += 1;
        let now = UTC::now().timestamp();
        let elapsed = now - start;

        if elapsed > 10 {
            break;
        }

        let mut y = 30;
        let mut line_height = 40;

        video.capture();
        if elapsed > 0 {
            video.draw_text(30, y, format!("Rendered {} frames in {} seconds", i+1, elapsed));
            y += line_height;
            video.draw_text(30, y, format!("FPS: {:.*}", 1, (i+1) / elapsed));
            y += line_height;
        }

        video.draw_text(30, y, match gps.get() {
            None => format!("GPS: N/A"),
            Some(loc) => format!("GPS: {:.*}, {:.*}", 6, loc.lat, 6, loc.lon)
        });
        y += line_height;

        video.draw_text(30, y, match compass.get() {
            None => format!("Compass: N/A"),
            Some(b) => format!("Compass: {:.*}", 1, b)
        });
        y += line_height;

        video.write();
    }

    video.close();
}

