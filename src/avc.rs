extern crate chrono;
extern crate navigation;

use super::video::*;
use super::qik::*;
use super::compass::*;
use super::gps::*;
use super::Config;

use chrono::UTC;
use navigation::*;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// the various actions the vehicle can be performing
#[derive(Debug, Clone, PartialEq)]
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
    bearing: Option<f64>,
    next_wp: Option<usize>,
    wp_bearing: Option<f64>,
    action: Option<Action>,
    speed: (i8,i8),
    finished: bool
}

impl State {

    fn new() -> Self {
        State { loc: None, bearing: None,
            next_wp: None, wp_bearing: None,
            action: None, speed: (0,0),
            finished: false }
    }

    fn set_action(&mut self, a: Action) {
        match self.action {
            None => println!("Action: {:?}", a),
            Some(ref b) => if &a != b {
                println!("Action: {:?}", a);
            }
        }
        self.action = Some(a);
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

fn navigate_to_waypoint(wp_num: usize, wp: &Location, io: &mut IO,
                        state: &mut State/*, tx: Sender<State>*/,
                        shared_state: &Arc<Mutex<Box<State>>>) {
    loop {

        // send a copy of the state to the video writer thread
//        tx.send(state.clone());

        {
            let mut x = shared_state.lock().unwrap();
            *x = Box::new(state.clone());
        }

        match io.gps.get() {
            None => {
                state.loc = None;
                state.set_action(Action::WaitingForGps);
                match io.qik {
                    None => {},
                    Some(ref mut q) => {
                        q.coast(Motor::M0);
                        q.coast(Motor::M1);
                        state.speed = (0,0);
                    }
                }
            },
            Some(loc) => {
                state.loc = Some((loc.lat, loc.lon));
                if close_enough(&loc, &wp) {
                    state.set_action(Action::ReachedWaypoint { waypoint: wp_num });
                    break;
                }

                match io.imu.get() {
                    None => {
                        state.bearing = None;
                        state.set_action(Action::WaitingForCompass);
                        match io.qik {
                            None => {},
                            Some(ref mut q) => {
                                q.coast(Motor::M0);
                                q.coast(Motor::M1);
                                state.speed = (0,0);
                            }
                        }
                    },
                    Some(b) => {
                        state.bearing = Some(b);
                        state.set_action(Action::Navigating { waypoint: wp_num });
                        let wp_bearing = loc.calc_bearing_to(&wp);
                        state.next_wp = Some(wp_num);
                        state.wp_bearing = Some(wp_bearing);
                        let turn_amount = calc_bearing_diff(b, wp_bearing);
                        match io.qik {
                            None => {},
                            Some(ref mut q) => {
                                //TODO: need real algorithms here
                                state.speed = if turn_amount < 0_f64 {
                                    (100,200)
                                } else {
                                    (200,100)
                                };
                                q.set_speed(Motor::M0, state.speed.0);
                                q.set_speed(Motor::M1, state.speed.1);
                            }
                        }
                    }
                }
            }
        };
    }
}

pub fn avc(conf: &Config, enable_motors: bool) {

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

    io.gps.start_thread();
    io.imu.start_thread();

    let shared_state = Arc::new(Mutex::new(Box::new(State::new())));

    // start the thread to write the video
    let video_state = shared_state.clone();
    let video_thread = thread::spawn(move || {
        let video = Video::new(0);
        let start = UTC::now().timestamp();
        video.init(format!("avc-{}.mp4", start)).unwrap();
        let mut i = 0;
        loop {
            i += 1;
            let now = UTC::now();
            let elapsed = now.timestamp() - start;

            // TEMP DEBUGGING
            if elapsed > 10 {
                break;
            }

            video.capture();
            {
                let s = video_state.lock().unwrap();

                println!("Frame {}: GPS={:?}, Compass={:?}, Next WP={:?}, WP_Bearing={:?}",
                         i, s.loc, s.bearing, s.next_wp, s.wp_bearing );

                if s.finished {
                    break;
                }

                let mut y = 30;
                let mut line_height = 30;

                // Time
                video.draw_text(30, y, format!("UTC: {}", now.format("%Y-%m-%d %H:%M:%S").to_string()));
                y += line_height;

                // FPS
                if elapsed > 0 {
                    video.draw_text(30, y, format!("FPS: {:.*}", 1, (i+1) / elapsed));
                    y += line_height;
                }

                // GPS
                video.draw_text(30, y, match s.loc {
                    None => format!("GPS: N/A"),
                    Some((lat,lon)) => format!("GPS: {:.*}, {:.*}", 6, lat, 6, lon)
                });
                y += line_height;

                // compass
                video.draw_text(30, y, match s.bearing {
                    None => format!("Compass: N/A"),
                    Some(b) => format!("Compass: {:.*}", 1, b)
                });
                y += line_height;

                // next waypoint number
                video.draw_text(30, y, match s.next_wp {
                    None => format!("Next WP: N/A"),
                    Some(wp) => format!("Next WP: {}", wp)
                });
                y += line_height;

                // bearing for next waypoint
                video.draw_text(30, y, match s.wp_bearing {
                    None => format!("WP Bearing: N/A"),
                    Some(b) => format!("WP Bearing: {}", b)
                });
                y += line_height;

                // action
                video.draw_text(30, y, match s.action {
                    Some(ref a) => format!("{:?}", a),
                    None => format!("")
                });

            }
            video.write();
            thread::sleep(Duration::from_millis(30));

        }
        video.close();
    });

    //TODO: wait for start button

    let mut state = State::new();
    let nav_state = shared_state.clone();
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);
        navigate_to_waypoint(i+1, &waypoint, &mut io, &mut state, &nav_state);
    }

    match io.qik {
        None => {},
        Some(ref mut q) => {
            q.set_brake(Motor::M0, 127);
            q.set_brake(Motor::M1, 127);
        }
    }

    // wait for video writer to finish
    video_thread.join().unwrap();

    println!("Finished");
}
