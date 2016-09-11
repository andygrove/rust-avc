extern crate chrono;
extern crate navigation;
extern crate graceful;

use super::video::*;
use super::compass::*;
use super::gps::*;
use super::motors::*;
use super::Config;
use super::switch::*;
use super::octasonic::*;

use chrono::UTC;
use chrono::DateTime;
use qik::*;
use navigation::*;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};

use self::graceful::SignalGuard;

static STOP: AtomicBool = ATOMIC_BOOL_INIT;

// NOTE: public fields are bad practice ... will fix later
pub struct Settings {
    pub max_speed: i8,
    pub differential_drive_coefficient: f32,
    pub enable_motors: bool,
    pub waypoints: Vec<Location>,
    pub obstacle_avoidance_distance: u8,
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
    Aborted,
    Finished,
}

/// instrumentation data to display on the video stream
#[derive(Clone,Debug)]
pub struct State {
    loc: Option<(f64, f64)>,
    bearing: Option<f32>,
    waypoint: Option<(usize, f32)>, // waypoint number and bearing
    turn: Option<f32>,
    pub action: Action,
    speed: (Motion, Motion),
    usonic: Vec<u8>,
}

impl State {
    fn new() -> Self {
        State {
            loc: None,
            bearing: None,
            waypoint: None,
            turn: None,
            action: Action::WaitingForStartCommand,
            speed: (Motion::Speed(0), Motion::Speed(0)),
            usonic: vec![255, 255, 255],
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
    motors: Motors<'a>,
}

pub struct AVC {
    conf: Config,
    settings: Settings,
    shared_state: Arc<Mutex<Box<State>>>,
}

impl AVC {
    pub fn new(conf: Config, settings: Settings) -> Self {
        AVC {
            conf: conf,
            settings: settings,
            shared_state: Arc::new(Mutex::new(Box::new(State::new()))),
        }
    }

    pub fn get_shared_state(&self) -> Arc<Mutex<Box<State>>> {
        self.shared_state.clone()
    }

    pub fn run(&self) {

        let mut qik = Qik::new(String::from(self.conf.qik_device), 0);

        let switch = Switch::new(17);

        let mut io = IO {
            gps: GPS::new(self.conf.gps_device),
            imu: Compass::new(self.conf.imu_device),
            motors: Motors::new(&mut qik, self.settings.enable_motors),
        };

        io.gps.start_thread();
        io.imu.start_thread();
        switch.start_thread();

        // sharing state with a Mutex rather than using channels due to the producer and consumer
        // operating at such different rates and the producer only needing the latest state 24
        // times per second
        //    let shared_state = Arc::new(Mutex::new(Box::new(State::new())));

        // because we want to see videos even if we CTRL-C ...
        let signal_guard = SignalGuard::new();

        let signal_state = self.shared_state.clone();
        let handle = thread::spawn(move || {
            println!("Worker thread started. Type Ctrl+C to stop.");
            while !STOP.load(Ordering::Acquire) {
                println!("working...");
                thread::sleep(Duration::from_millis(500));
            }
            let mut state = signal_state.lock().unwrap();
            state.set_action(Action::Aborted);
            println!("Bye.");
        });

        signal_guard.at_exit(move |sig| {
            println!("Signal {} received.", sig);
            STOP.store(true, Ordering::Release);
            handle.join().unwrap();
        });

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
                    // println!("{:?}", *s);

                    // stop capturing video at end of race
                    match s.action {
                        Action::Aborted | Action::Finished => {
                            println!("Aborting video writer thread");
                            break;
                        }
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

        // wait for start switch
        println!("Waiting for START switch...");
        loop {
            match switch.get() {
                Some(true) => break,
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }
        println!("Detected START switch!");


        let nav_state = self.shared_state.clone();
        for (i, waypoint) in self.settings.waypoints.iter().enumerate() {
            if !self.navigate_to_waypoint(i + 1,
                                          &waypoint,
                                          &mut io,
                                          &mut state,
                                          &nav_state,
                                          &o,
                                          &switch) {

                // set shared state to Aborted so the video thread finishes
                let mut state = nav_state.lock().unwrap();
                state.set_action(Action::Aborted);
                break;
            }
        }

        // set action to finished, unless it is Aborted
        match state.action {
            Action::Aborted => {}
            _ => {
                let mut state = nav_state.lock().unwrap();
                state.set_action(Action::Finished);
            },
        }

        // we'd better stop now
        io.motors.set(Motion::Brake(127), Motion::Brake(127));

        // wait for video writer to finish
        println!("nav thread waiting for video thread to terminate");
        video_thread.join().unwrap();
        println!("nav thread finished waiting for video thread to terminate");
    }

    fn navigate_to_waypoint(&self,
                            wp_num: usize,
                            wp: &Location,
                            io: &mut IO,
                            state: &mut State,
                            nav_state: &Arc<Mutex<Box<State>>>,
                            o: &Octasonic,
                            switch: &Switch)
                            -> bool {
        loop {

            match switch.get() {
                Some(false) => return false,
                _ => {}
            }

            // replace the shared state ... using a block here to limit the scope of the mutex
            {
                let mut x = nav_state.lock().unwrap();

                match x.action {
                    Action::Aborted => {
                        println!("Aborting navigation to waypoint {}", wp_num);
                        return false;
                    }
                    _ => {}
                };

                *x = Box::new(state.clone());
            }

            // performing this logic 100 times per second should be enough
            thread::sleep(Duration::from_millis(10));

            match self.check_for_obstacles(state, o) {
                Some(a) => {
                    state.speed = match a {
                        Action::AvoidingObstacleToLeft => {
                            (Motion::Speed(self.settings.max_speed), Motion::Speed(0))
                        }
                        Action::AvoidingObstacleToRight => {
                            (Motion::Speed(0), Motion::Speed(self.settings.max_speed))
                        }
                        Action::EmergencyStop => (Motion::Brake(127), Motion::Brake(127)),
                        _ => panic!("Unsupported avoidance action: {:?}", a),
                    };
                    state.action = a;
                    io.motors.set(state.speed.0, state.speed.1);
                    continue;
                }
                None => {}
            }

            match io.gps.get() {
                None => {
                    state.loc = None;
                    state.set_action(Action::WaitingForGps);
                    let s = (Motion::Speed(0), Motion::Speed(0));
                    io.motors.set(s.0, s.1);
                    state.speed = s;
                }
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
                            let s = (Motion::Speed(0), Motion::Speed(0));
                            io.motors.set(s.0, s.1);
                            state.speed = s;
                        }
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
                            state.speed = (Motion::Speed(left_speed), Motion::Speed(right_speed));
                            io.motors.set(state.speed.0, state.speed.1);
                        }
                    }
                }
            };
        }
    }

    fn check_for_obstacles(&self, state: &mut State, o: &Octasonic) -> Option<Action> {

        for i in 0..3 {
            state.usonic[i] = o.get_sensor_reading(i as u8);
        }

        let fl = state.usonic[2];
        let ff = state.usonic[1];
        let fr = state.usonic[0];

        if ff < self.settings.obstacle_avoidance_distance {
            if fl < self.settings.obstacle_avoidance_distance &&
               fr < self.settings.obstacle_avoidance_distance {
                Some(Action::EmergencyStop)
            } else if fl < fr {
                Some(Action::AvoidingObstacleToLeft)
            } else {
                Some(Action::AvoidingObstacleToRight)
            }
        } else if fl < self.settings.obstacle_avoidance_distance {
            Some(Action::AvoidingObstacleToLeft)
        } else if fr < self.settings.obstacle_avoidance_distance {
            Some(Action::AvoidingObstacleToRight)
        } else {
            None
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
    } else if ret > 180_f32 {
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

    let x1 = 30;
    let x2 = 350;
    let top = 20;
    let line_height = 20;
    let mut y = top + line_height;

    let c = Color::new(200, 200, 200, 24); // r, g, b, alpha
    let background = Color::new(50, 50, 50, 24); // r, g, b, alpha

    video.fill_rect(top, 20, 600, top + line_height * 5, &background);

    // COLUMN 1

    // GPS
    video.draw_text(x1,
                    y,
                    match s.loc {
                        None => format!("GPS: N/A"),
                        Some((lat, lon)) => format!("GPS: {:.*}, {:.*}", 6, lat, 6, lon),
                    },
                    &c);
    y += line_height;

    // compass
    video.draw_text(x1,
                    y,
                    match s.bearing {
                        None => format!("Compass: N/A"),
                        Some(b) => format!("Compass: {:.*}", 1, b),
                    },
                    &c);
    y += line_height;

    // next waypoint number
    video.draw_text(x1,
                    y,
                    match s.waypoint {
                        None => format!("Waypoint: N/A"),
                        Some((n, b)) => format!("Waypoint: {} @ {:.*}", n, 1, b),
                    },
                    &c);
    y += line_height;

    // how much do we need to turn?
    video.draw_text(x1,
                    y,
                    match s.turn {
                        None => format!("Turn: N/A"),
                        Some(b) => format!("Turn: {:.*}", 1, b),
                    },
                    &c);
    y += line_height;

    // action
    video.draw_text(x1, y, format!("{:?}", s.action), &c);

    // COLUMN 2

    y = top + line_height;

    // Date
    video.draw_text(x2,
                    y,
                    format!("UTC: {}", now.format("%Y-%m-%d %H:%M:%S").to_string()),
                    &c);
    y += line_height;

    // FPS
    if elapsed > 0 {
        let fps: f32 = (frame as f32) / (elapsed as f32);
        video.draw_text(x2, y, format!("FPS: {:.*}", 1, fps), &c);
    } else {
        video.draw_text(x2, y, String::from("FPS: N/A"), &c);
    }
    y += line_height;

    // motor speeds
    video.draw_text(x2,
                    y,
                    format!("Motors: {:?} / {:?}", s.speed.0, s.speed.1),
                    &c);
    y += line_height;

    // ultrasonic sensors
    video.draw_text(x2,
                    y,
                    format!("FL={}, FF={}, FR={}", s.usonic[2], s.usonic[1], s.usonic[0]),
                    &c);
    y += line_height;

}
