extern crate hyper;
extern crate iron;
extern crate urlencoded;
extern crate getopts;
extern crate chrono;
extern crate navigation;
extern crate qik;
extern crate yaml_rust;
extern crate sysfs_gpio;

use sysfs_gpio::{Direction, Pin};
use getopts::Options;
use chrono::UTC;
use navigation::*;
use qik::*;
use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedQuery;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use yaml_rust::{YamlLoader, Yaml};

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

mod gps;
mod compass;
mod video;
mod avc;
mod motors;
mod switch;

use gps::GPS;
use compass::Compass;
use video::*;
use avc::*;
use switch::*;

mod octasonic;
use octasonic::*;

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
    opts.optflag("s", "test-switch", "tests the switch");
    opts.optflag("u", "test-ultrasonic", "tests the ultrasonic sensors");
    opts.optflag("w", "test-ultrasonic-with-motors", "tests the ultrasonic sensors and motors together");
    opts.optflag("c", "capture-gps", "records a GPS waypoint to file");
    opts.optflag("a", "avc", "Start the web server");
    opts.optopt("f", "filename", "Course filename", "conf/avc.yaml");

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
    else if matches.opt_present("s") { test_switch(); }
    else if matches.opt_present("u") { test_ultrasonic(); }
    else if matches.opt_present("w") { test_ultrasonic_with_motors(&conf); }
    else if matches.opt_present("c") { capture_gps(&conf); }
    else if matches.opt_present("a") {
        let filename = matches.opt_str("f").unwrap();
        run_avc(conf, &filename);
    }
    else { panic!("missing cmd line argument .. try --help"); }

}

fn run_avc(conf: Config, filename: &str) {

    let mut input = String::new();
    let mut file = File::open(filename).unwrap();
    file.read_to_string(&mut input).unwrap();
    let docs = YamlLoader::load_from_str(&input).unwrap();
    let doc = &docs[0].as_hash().unwrap();

    let waypoints = doc.get(&Yaml::String(String::from("waypoints"))).unwrap().as_vec().unwrap();
    let mut course : Vec<Location> = vec![];
    for i in 0 .. waypoints.len() {
        let wp = &waypoints[i].as_vec().unwrap();
        let lat = wp[0].as_f64().unwrap();
        let lon = wp[1].as_f64().unwrap();
        println!("wp {} = {:?}, {:?}", i, lat, lon);
        course.push(Location::new(lat, lon));
    }

    let settings = Settings {
        enable_motors: true, //doc.get(&Yaml::String(String::from("enable_motors"))).unwrap().as_bool().unwrap(),
        max_speed: doc.get(&Yaml::String(String::from("max_speed"))).unwrap().as_i64().unwrap() as i8,
        obstacle_avoidance_distance: doc.get(&Yaml::String(String::from("obstacle_avoidance_distance"))).unwrap().as_i64().unwrap() as u8,
        differential_drive_coefficient: 1_f32,
        waypoints: course
    };

    let avc = AVC::new(conf, settings);

    let start_state = avc.get_shared_state();
    let web_state = avc.get_shared_state();

    let _ = thread::spawn(move || {

        // wait for the user to hit the start button
        println!("Waiting for start command");
        loop {
            {
                let state = start_state.lock().unwrap();
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

//        let mut headers = Headers::new();
//        headers.set(
//            ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))
//        );

        let css = ".start_button {\
                    background-color: green;\
                    border: none;\
                    color: white;\
                    padding: 15px 32px;\
                    text-align: center;\
                    text-decoration: none;\
                    display: inline-block;\
                    font-size: 48px;\
                }\
                \
                .stop_button {\
                    background-color: red;\
                    border: none;\
                    color: white;\
                    padding: 15px 32px;\
                    text-align: center;\
                    text-decoration: none;\
                    display: inline-block;\
                    font-size: 48px;\
                }";

        let start_page = format!("<html>
                            <head><style>{}</style></head><body>
                            <form action=\"/\">\
                            <input type=\"hidden\" name=\"action\" value=\"start\">\
                            <input type=\"submit\" value=\"Start!\" class=\"start_button\">\
                            </form></body></html>", css);

        let stop_page = format!("<html>
                            <head><style>{}</style></head><body>
                            <form action=\"/\">\
                            <input type=\"hidden\" name=\"action\" value=\"stop\">\
                            <input type=\"submit\" value=\"Stop!\" class=\"stop_button\">\
                            </form>
                            <a href=\"/\">HOME</a>\
                            </body></html>", css);

        let restart_page = format!("<html>
                            <head><style>{}</style></head><body>\
                            <h1>Stopped! You will need to restart the cmd-line app before starting again!</h1>\
                            <form action=\"/\">\
                            <input type=\"hidden\" name=\"action\" value=\"start\">\
                            <input type=\"submit\" value=\"Start!\" class=\"start_button\">\
                            </form></body></html>", css);

        let invalid_action = format!("<html>
                            <head><style>{}</style></head><body>\
                            <h1>Invalid action!</h1>\
                            <a href=\"/\">HOME</a>\
                            </body></html>", css);

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
                                let mut r = Response::with((status::Ok, stop_page));
                                r.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
                                Ok(r)
                            },
                            "stop" => {
                                {
                                    let mut state = web_state.lock().unwrap();
                                    state.set_action(Action::Aborted);
                                }
                                let mut r = Response::with((status::Ok, restart_page));
                                r.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
                                Ok(r)
                            },
                            _ => {
                                let mut r = Response::with((status::Ok, invalid_action));
                                r.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
                                Ok(r)
                            }
                        }
                    },
                    None => {
                        let mut r = Response::with((status::Ok, start_page));
                        r.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
                        Ok(r)
                    }
                }

            },
            Err(ref e) => {
                println!("Error: {:?}", e);
                let mut r = Response::with((status::Ok, start_page));
                r.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
                Ok(r)
            }
        }

    }).http("0.0.0.0:8080").unwrap();

}

fn capture_gps(conf: &Config) {
    println!("Capturing GPS");
    let gps = GPS::new(conf.gps_device);
    gps.start_thread();
    loop {
        if let Some(wp) = gps.get() {
            println!("Captured: {:.*}, {:.*}", 6, wp.lat, 6, wp.lon);

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("captured-waypoints.txt").unwrap();

            // write out in YAML format ready for copy-and-paste
            let s = format!("  - [{:.*}, {:.*}] # captured at {:?}\n", 6, wp.lat, 6, wp.lon, UTC::now());
            let b = &s.as_ref();
            file.write(b).unwrap();

            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
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


    let c = Color::new(200,200,200,24); // r, g, b, alpha
    let background = Color::new(50,50,50,24); // r, g, b, alpha

    let mut i = 0;
    loop {
        i += 1;
        let now = UTC::now().timestamp();
        let elapsed = now - start;

        if elapsed > 10 {
            break;
        }

        let mut y = 30;
        let line_height = 20;

        video.capture();

        video.fill_rect(10, 10, 620, 150, &background);
        
        if elapsed > 0 {
            video.draw_text(30, y, format!("Rendered {} frames in {} seconds", i+1, elapsed), &c);
            y += line_height;
            video.draw_text(30, y, format!("FPS: {:.*}", 1, (i+1) / elapsed), &c);
            y += line_height;
        }

        video.draw_text(30, y, match gps.get() {
            None => format!("GPS: N/A"),
            Some(loc) => format!("GPS: {:.*}, {:.*}", 6, loc.lat, 6, loc.lon)
        }, &c);
        y += line_height;

        video.draw_text(30, y, match compass.get() {
            None => format!("Compass: N/A"),
            Some(b) => format!("Compass: {:.*}", 1, b)
        }, &c);

        video.write();
    }

    video.close();
}

fn test_ultrasonic() {
    let o = Octasonic::new();
    let n = 3; // sensor count
    o.set_sensor_count(n);
    let m = o.get_sensor_count();
    if n != m {
        panic!("Warning: failed to set sensor count! {} != {}", m, n);
    }

    let mut b = true;
    loop {
        print!("Ultrasonic: ");
        for i in 0..n {
            print!("{}  ", o.get_sensor_reading(i));
        }
        println!(" cm");
        thread::sleep(Duration::from_millis(100));
    }
}

fn test_ultrasonic_with_motors(conf: &Config) {
    let o = Octasonic::new();
    let n = 3; // sensor count
    o.set_sensor_count(n);
    let m = o.get_sensor_count();
    if n != m {
        panic!("Warning: failed to set sensor count! {} != {}", m, n);
    }

    use qik::ConfigParam::*;
    let mut qik = qik::Qik::new(String::from(conf.qik_device), 123);
    qik.init();

    let mut counter = 0;
    let mut b = true;
    loop {
        print!("Ultrasonic: ");
        for i in 0..n {
            print!("{}  ", o.get_sensor_reading(i));
        }
        println!(" cm");
        thread::sleep(Duration::from_millis(100));
        counter += 1;
        if counter % 10 == 0 {
            counter = 0;
            b = !b;
            if b {
                qik.set_speed(Motor::M0, 80);
                qik.set_speed(Motor::M1, 80);
            } else {
                qik.set_speed(Motor::M0, 0);
                qik.set_speed(Motor::M1, 0);
            }
        }
    }
}

fn test_switch() {
  println!("Testing switch");
  let mut s = Switch::new(17);
  s.start_thread();
  let mut state = s.get();
    loop {
      let new_state = s.get();
      if state != new_state {
        println!("Switch is {:?}", new_state);
        state = new_state;
      }
    }
 
      
}


