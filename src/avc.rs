extern crate chrono;
extern crate navigation;

use super::video::*;
use super::qik::*;
use super::compass::*;
use super::gps::*;
use super::Config;

use chrono::*;
use navigation::*;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// the various actions the vehicle can be performing
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

/// instrumentation data to display on the video stream
#[derive(Clone,Debug)]
pub struct State {
    loc: Option<(f64,f64)>,
    bearing: Option<f32>,
    next_wp: Option<u8>,
    wp_bearing: Option<f32>,
    action: Option<String>
}

impl State {

    fn new() -> Self {
        State { loc: None, bearing: None, next_wp: None, wp_bearing: None, action: None }
    }

    fn set_action(&mut self, a: Action) {
        println!("Action: {:?}", a);
        self.action = Some(format!("{:?}", a));

    }

}

/// group all the IO devices in a single strut to make it easier to pass them around
struct IO<'a> {
    gps: GPS,
    imu: Compass,
    qik: Option<Qik>,
    video: &'a Video
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

fn navigate_to_waypoint(wp_num: usize, wp: &Location, io: &mut IO, state: &mut State) {
    state.set_action(Action::Navigating { waypoint: wp_num });
    loop {
        match io.gps.get() {
            None => {
                state.set_action(Action::WaitingForGps);
                match io.qik {
                    None => {},
                    Some(ref mut q) => {
                        q.coast(Motor::M0);
                        q.coast(Motor::M1);
                    }
                }
            },
            Some(loc) => {
                if close_enough(&loc, &wp) {
                    state.set_action(Action::ReachedWaypoint { waypoint: wp_num });
                    break;
                }

                match io.imu.get() {
                    None => {
                        state.set_action(Action::WaitingForCompass);
                        match io.qik {
                            None => {},
                            Some(ref mut q) => {
                                q.coast(Motor::M0);
                                q.coast(Motor::M1);
                            }
                        }
                    },
                    Some(b) => {
                        let wp_bearing = loc.calc_bearing_to(&wp);
                        let turn_amount = calc_bearing_diff(b, wp_bearing);
                        match io.qik {
                            None => {},
                            Some(ref mut q) => {
                                if turn_amount < 0_f64 {
                                    q.set_speed(Motor::M0, 100);
                                    q.set_speed(Motor::M1, 200);
                                } else {
                                    q.set_speed(Motor::M0, 200);
                                    q.set_speed(Motor::M1, 100);
                                }
                            }
                        }
                    }
                }
            }
        };
    }
}

fn instrument_video(v: &Video, s: &State) {

    match s.action {
        Some(ref a) => v.draw_text(100, 100, format!("{:?}", a)),
        None => {}
    }

}

struct VideoWriter {
    state: Arc<Mutex<State>>
}

impl VideoWriter {

    /// logic
    fn record(&self) {
    }
}

pub fn avc(conf: &Config, enable_motors: bool) {

    let mut state = State::new();

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.94177796143009, -105.08160397410393),
        Location::new(39.94190648894769, -105.08158653974533),
        Location::new(39.94186741660787, -105.08174613118172),
    ];

    let video = Video::new(0);

    let mut io = IO {
        gps: GPS::new(conf.gps_device),
        imu: Compass::new(conf.imu_device),
        qik: if enable_motors { Some(Qik::new(String::from(conf.qik_device), 0)) } else { None },
        video: &video
    };

    // set up a channel to send state info to video writer
    //let (tx, rx): (Sender<State>, Receiver<State>) = mpsc::channel();

    let state = Arc::new(Mutex::new(State::new));

    // start the thread to write the video
    let video_state = state.clone();
    let video_thread = thread::spawn(move || {
        let video = Video::new(0);
        let start = UTC::now().timestamp();
        video.init(format!("avc-{}.mp4", start)).unwrap();
        loop {
            video.capture();
//            {
//                let state = video_state.lock().unwrap();
//                match state {
//                    Finished => break,
//                    _ => {}
//                }
//                instrument_video(&video, &state);
//            }
            video.write();

        }
        video.close();
    });

    //TODO: wait for start button

    // navigate to each waypoint in turn
    let mut state = State::new();
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);
//        let thread_tx = tx.clone();
//        thread_tx.send(state.clone()).unwrap();
        navigate_to_waypoint(i+1, &waypoint, &mut io, &mut state);
    }


//    match io.qik {
//        None => {},
//        Some(q) => {
//            q.set_brake(Motor::M0, 127);
//            q.set_brake(Motor::M1, 127);
//        }
//    }

    // wait for video writer to finish
    video_thread.join().unwrap();

    println!("Finished");
}
