extern crate chrono;
extern crate navigation;

use super::video::*;
use super::compass::*;
use super::gps::*;
use super::octasonic::*;
use super::Config;

use chrono::UTC;
use chrono::DateTime;
use qik::*;
use navigation::*;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

//NOTE: public fields are bad practice ... will fix later
pub struct Settings {
    pub max_speed: i8,
    pub differential_drive_coefficient: f32,
    pub enable_motors: bool,
    pub waypoints: Vec<Location>
}

/// the various actions the vehicle can be performing
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    WaitingForStartCommand,
    Navigating { waypoint: usize },
    ReachedWaypoint { waypoint: usize },
    WaitingForGps,
    WaitingForCompass,
    AvoidingObstacleToLeft,
    AvoidingObstacleToRight,
    EmergencyStop,
    Finished,
    Aborted
}

/// instrumentation data to display on the video stream
#[derive(Clone,Debug)]
pub struct State {
    loc: Option<(f64,f64)>,
    bearing: Option<f32>,
    waypoint: Option<(usize, f32)>, // waypoint number and bearing
    turn: Option<f32>,
    pub action: Action,
    speed: (i8,i8),
    usonic: Vec<u8>
}

impl State {

    fn new() -> Self {
        State {
            loc: None,
            bearing: None,
            waypoint: None,
            turn: None,
            action: Action::WaitingForStartCommand,
            speed: (0,0),
            usonic: vec![255,255,255]
        }
    }

    pub fn set_action(&mut self, a: Action) {
        if self.action != a {
            println!("Action: {:?}", a);
        }
        self.action = a;
    }

}

/// group all the IO devices in a single strut to make it easier to pass them around
struct IO<'a> {
    gps: GPS,
    imu: Compass,
    qik: Option<Qik>,
    video: &'a Video
}

pub struct AVC {
    conf: Config,
    settings: Settings,
    shared_state: Arc<Mutex<Box<State>>>
}

impl AVC {

    pub fn new(conf: Config, settings: Settings) -> Self {
        AVC {
            conf: conf,
            settings: settings,
            shared_state: Arc::new(Mutex::new(Box::new(State::new())))
        }
    }

    pub fn get_shared_state(&self) -> Arc<Mutex<Box<State>>> {
        self.shared_state.clone()
    }

    pub fn run(&self) {

        let video = Video::new(0);

        let mut io = IO {
            gps: GPS::new(self.conf.gps_device),
            imu: Compass::new(self.conf.imu_device),
            qik: if self.settings.enable_motors {
                Some(Qik::new(String::from(self.conf.qik_device), 0))
            } else {
                None
            },
            video: &video
        };

        io.gps.start_thread();
        io.imu.start_thread();

        // sharing state with a Mutex rather than using channels due to the producer and consumer
        // operating at such different rates and the producer only needing the latest state 24
        // times per second
        //    let shared_state = Arc::new(Mutex::new(Box::new(State::new())));

        // start the thread to write the video
        let video_state = self.shared_state.clone();
        let video_thread = thread::spawn(move || {
            let video = Video::new(0);
            let start = UTC::now().timestamp();
            let filename = format!("avc-{}.mp4", start);
            println!("Writing video to {}", filename);
            video.init(filename).unwrap();
            let mut frame = 0;
            loop {
                frame += 1;

                let now = UTC::now();
                let elapsed = now.timestamp() - start;

                video.capture();

                {
                    let s = video_state.lock().unwrap();
                    //println!("{:?}", *s);

                    // stop capturing video at end of race
                    match s.action {
                        Action::Finished | Action::Aborted => {
                            println!("Aborting video writer thread");
                            break;
                        },
                        _ => {}
                    };

                    augment_video(&video, &s, now, elapsed, frame);
                }

                video.write();
            }

            println!("Closing video file");
            video.close();
        });

        let o = Octasonic::new();
        let n = 3; // sensor count
        o.set_sensor_count(n);
        let m = o.get_sensor_count();
        if n != m {
            panic!("Warning: failed to set sensor count! {} != {}", m, n);
        }

        let mut state = State::new();
        let nav_state = self.shared_state.clone();
        for (i, waypoint) in self.settings.waypoints.iter().enumerate() {
            if !self.navigate_to_waypoint(i+1, &waypoint, &mut io, &mut state, &nav_state, &o) {
                break;
            }
        }

        match io.qik {
            None => {},
            Some(ref mut q) => {
                q.set_brake(Motor::M0, 127);
                q.set_brake(Motor::M1, 127);
            }
        }

        // wait for video writer to finish
        println!("nav thread waiting for video thread to terminate");
        video_thread.join().unwrap();
        println!("nav thread finished waiting for video thread to terminate");
    }

    fn navigate_to_waypoint(&self, wp_num: usize, wp: &Location, io: &mut IO,
                            state: &mut State,
                            nav_state: &Arc<Mutex<Box<State>>>,
                            o: &Octasonic
    ) -> bool {
        loop {

            // replace the shared state ... using a block here to limit the scope of the mutex
            {
                let mut x = nav_state.lock().unwrap();

                match x.action {
                    Action::Finished | Action::Aborted => {
                        println!("Aborting navigation to waypoint {}", wp_num);
                        return false;
                    },
                    _ => {}
                };

                *x = Box::new(state.clone());
            }

            // performing this logic 100 times per second should be enough
            thread::sleep(Duration::from_millis(10));

            // check for obstacles
            for i in 0 .. 3 {
                state.usonic[i] = o.get_sensor_reading(i as u8);
            }

            let fl = state.usonic[2];
            let ff = state.usonic[1];
            let fr = state.usonic[0];

let x = 20;

            let avoid = if fl < x {
                            Some(Action::AvoidingObstacleToLeft)
                        } else if fr < x {
                            Some(Action::AvoidingObstacleToRight)
                        } else if ff < 50 {
                            if fl < x && fr < x {
                                Some(Action::EmergencyStop)
                            } else if fl < fr {
                                Some(Action::AvoidingObstacleToLeft)
                            } else {
                                Some(Action::AvoidingObstacleToRight)
                            }
                        } else {
                            None
                        };

            match avoid {
                Some(a) => {
                    state.speed = match a {
                        Action::AvoidingObstacleToLeft => (self.settings.max_speed, 0),
                        Action::AvoidingObstacleToRight => (0, self.settings.max_speed),
                        Action::EmergencyStop => (0, 0),
                        _ => panic!("Unsupported avoidance action: {:?}", a)
                    };
                    state.action = a;
                    match io.qik {
                        None => {},
                        Some(ref mut q) => {
                            q.set_speed(Motor::M0, state.speed.0);
                            q.set_speed(Motor::M1, state.speed.1);
                        }
                    }
                    continue
                },
                None => {}
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
                        return true;
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
                            state.set_action(Action::Navigating { waypoint: wp_num });
                            let wp_bearing = loc.calc_bearing_to(&wp) as f32;
                            let turn = calc_bearing_diff(b, wp_bearing);
                            let mut left_speed = self.settings.max_speed;
                            let mut right_speed = self.settings.max_speed;

                            if turn < 0_f32 {
                                // turn left by reducing speed of left motor
                                left_speed = calculate_motor_speed(&self.settings, turn.abs());
                            } else {
                                // turn right by reducing speed of right motor
                                right_speed = calculate_motor_speed(&self.settings, turn.abs());
                            };

                            state.bearing = Some(b);
                            state.waypoint = Some((wp_num, wp_bearing));
                            state.turn = Some(turn);
                            state.speed = (left_speed, right_speed);

                            match io.qik {
                                None => {},
                                Some(ref mut q) => {
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
}


fn close_enough(a: &Location, b: &Location) -> bool {
    (a.lat - b.lat).abs() < 0.000025 && (a.lon - b.lon).abs() < 0.000025
}

fn calc_bearing_diff(current_bearing: f32, wp_bearing: f32) -> f32 {
    let mut ret = wp_bearing - current_bearing;
    if ret < -180_f32 {
        ret += 360_f32;
    }
        else if ret > 180_f32 {
            ret -= 360_f32;
        }
    ret
}



/// Calculate motor speed based on angle of turn.
fn calculate_motor_speed(settings: &Settings, angle: f32) -> i8 {
    let mut temp = angle * settings.differential_drive_coefficient;
    if temp > 180_f32 {
        temp = 180_f32;
    }
    let coefficient = (180_f32 - temp) / 180_f32;
    (coefficient * (settings.max_speed as f32)) as i8
}


fn augment_video(video: &Video, s: &State, now: DateTime<UTC>, elapsed: i64, frame: i64) {

    let mut y = 30;
    let mut line_height = 25;

    // FPS
    if elapsed > 0 {
        let fps : f32 = (frame as f32) / (elapsed as f32);
        video.draw_text(500, 25, format!("FPS: {:.*}", 1, fps));
    }

    // Time
    video.draw_text(30, y, format!("UTC: {}", now.format("%Y-%m-%d %H:%M:%S").to_string()));
    y += line_height;

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
    video.draw_text(30, y, match s.waypoint {
        None => format!("Waypoint: N/A"),
        Some((n,b)) => format!("Waypoint: {} @ {:.*}", n, 1, b)
    });
    y += line_height;

    // how much do we need to turn?
    video.draw_text(30, y, match s.turn {
        None => format!("Turn: N/A"),
        Some(b) => format!("Turn: {:.*}", 1, b)
    });
    y += line_height;

    // motor speeds
    video.draw_text(30, y, format!("Motors: {} / {}", s.speed.0, s.speed.1));
    y += line_height;

    // ultrasonic sensors
    video.draw_text(30, y, format!("Ultrasonic: {} {} {}", s.usonic[0], s.usonic[1], s.usonic[2]));
    y += line_height;

    // action
    video.draw_text(30, y, format!("{:?}", s.action));

}



