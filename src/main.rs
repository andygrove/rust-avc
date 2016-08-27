
extern crate getopts;
extern crate chrono;
extern crate navigation;

use getopts::Options;

use chrono::UTC;

use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod gps;
mod compass;
mod motors;
mod video;
mod qik;
mod avc;

use navigation::*;
use gps::GPS;
use compass::Compass;
use motors::Motors;
use video::Video;
use qik::*;
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
    opts.optflag("a", "avc", "navigate a course, for real!");

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
    else if matches.opt_present("t") { avc(&conf, false); }
    else if matches.opt_present("a") { avc(&conf, true); }
    else { panic!("missing cmd line argument .. try --help"); }

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
    let mut qik = qik::Qik::new(conf.qik_device, 123);
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

        if elapsed > 30 {
            break;
        }

        video.capture();
        if elapsed > 0 {
            video.draw_text(100, 30, format!("Rendered {} frames in {} seconds", i+1, elapsed));
            video.draw_text(100, 50, format!("FPS: {:.*}", 1, (i+1) / elapsed));
        }

        video.draw_text(30, 30, match gps.get() {
            None => format!("GPS: N/A"),
            Some(loc) => format!("GPS: {}, {}", loc.lat, loc.lon)
        });

        video.draw_text(30, 50, match compass.get() {
            None => format!("Compass: N/A"),
            Some(b) => format!("Compass: {}", b)
        });

        video.write();
    }

    video.close();
}

