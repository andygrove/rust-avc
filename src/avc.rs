extern crate chrono;
extern crate navigation;
extern crate graceful;

use super::video::*;
use super::compass::*;
use super::lidar::*;
use super::gps::*;
use super::motors::*;
use super::Config;
use super::switch::*;

use chrono::UTC;
use chrono::DateTime;
use qik::*;
use navigation::*;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// NOTE: public fields are bad practice ... will fix later
pub struct Settings {
    pub max_speed: i8,
    pub differential_drive_coefficient: f32,
    pub waypoint_accuracy: (f64, f64), // lat, lon
    pub waypoints: Vec<Location>,
    pub obstacle_avoidance_distance: u32,
    pub usonic_sample_count: usize,
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
    /// Waypoint number and location (lat, lon)
    next_waypoint: Option<(usize, (f64,f64))>,
    waypoint_bearing: Option<f32>,
    turn: Option<f32>,
    pub action: Action,
    speed: (Motion, Motion),
    lidar: Vec<u32>,
    distance_front: u32,
    distance_front_left: u32,
    distance_front_right: u32,
    distance_side_left: u32,
    distance_side_right: u32,
}

impl State {
    fn new() -> Self {
        State {
            loc: None,
            bearing: None,
            next_waypoint: None,
            waypoint_bearing: None,
            turn: None,
            action: Action::WaitingForStartCommand,
            speed: (Motion::Speed(0), Motion::Speed(0)),
            lidar: vec![0_u32; 360],
            distance_front: 0,
            distance_front_left: 0,
            distance_front_right: 0,
            distance_side_left: 0,
            distance_side_right: 0,
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
    lidar: Lidar
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

    pub fn run(&self) {

        let mut qik = Qik::new(String::from(self.conf.qik_device), 18).unwrap();
        qik.init().unwrap();

        let switch = Switch::new(17);

        let mut io = IO {
            gps: GPS::new(self.conf.gps_device),
            imu: Compass::new(self.conf.imu_device),
            motors: Motors::new(&mut qik),
            lidar: Lidar::new(String::from(self.conf.lidar_device))
        };

        io.gps.start_thread();
//        io.imu.start_thread().unwrap();
        switch.start_thread();

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
            println!("Video thread terminated");
        });

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
            }
        }

        // we'd better stop now
        io.motors.set(Motion::Brake(127), Motion::Brake(127));

        // wait for video writer to finish
        println!("Waiting for video thread to terminate ...");
        video_thread.join().unwrap();
        println!("Finished!");
    }

    fn navigate_to_waypoint(&self,
                            wp_num: usize,
                            wp: &Location,
                            io: &mut IO,
                            state: &mut State,
                            nav_state: &Arc<Mutex<Box<State>>>,
                            switch: &Switch)
                            -> bool {

        println!("navigate_to_waypoint({})", wp_num);

        // update next_waypoint
        state.next_waypoint = Some((wp_num, (wp.lat, wp.lon)));

        loop {

            // check for kill switch
            match switch.get() {
                Some(false) => return false,
                _ => {}
            }

            // update shared state so video can record latest data, and return if the
            // shared state says to abort
            if !self.update_shared_state(state, nav_state) {
                return false;
            }

            // give the CPU a breather and let some other threads run
            thread::sleep(Duration::from_millis(10));

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
                    if self.close_enough(&loc, &wp) {
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
                            state.bearing = Some(b);

                            // simulate ultrasonic sensors with LIDAR data
                            state.distance_front_left  = io.lidar.min(225, 315);
                            state.distance_front       = io.lidar.min(315, 45);
                            state.distance_front_right = io.lidar.min(45, 135);
                            io.lidar.get(&mut state.lidar);

                            match self.check_obstacles(&state) {
                                Some(avoid) => {
                                    match avoid {
                                        Action::AvoidingObstacleToLeft => {
                                            state.set_action(avoid);
                                            state.turn = None;
                                            state.speed = (Motion::Speed(self.settings.max_speed), Motion::Speed(0));
                                        },
                                        Action::AvoidingObstacleToRight => {
                                            state.set_action(avoid);
                                            state.turn = None;
                                            state.speed = (Motion::Speed(0), Motion::Speed(self.settings.max_speed));
                                        },
                                        Action::EmergencyStop => {
                                            state.set_action(avoid);
                                            state.turn = None;
                                            state.speed = (Motion::Brake(127), Motion::Brake(127));
                                        },
                                        _ => {
                                            println!("Invalid avoidance action");
                                        }
                                    };
                                },
                                None => {
                                    // continue with navigation towards waypoint
                                    state.set_action(Action::Navigating { waypoint: wp_num });

                                    let wp_bearing = loc.calc_bearing_to(&wp) as f32;

                                    let turn = calc_bearing_diff(b, wp_bearing);
                                    let mut left_speed = self.settings.max_speed;
                                    let mut right_speed = self.settings.max_speed;

                                    if turn < 0_f32 {
                                        // turn left by reducing speed of left motor
                                        left_speed = calculate_motor_speed(&self.settings,
                                                                           turn.abs());
                                    } else {
                                        // turn right by reducing speed of right motor
                                        right_speed = calculate_motor_speed(&self.settings,
                                                                            turn.abs());
                                    }

                                    state.waypoint_bearing = Some(wp_bearing);
                                    state.turn = Some(turn);
                                    state.speed = (Motion::Speed(left_speed),
                                                   Motion::Speed(right_speed));
                                }
                            }

                            // set motor speeds
                            io.motors.set(state.speed.0, state.speed.1);
                        }
                    }
                }
            };
        }
    }

    /// check sensors and determine if we need to take some action
    fn check_obstacles(&self, state: &State) -> Option<Action> {

        let min_d = self.settings.obstacle_avoidance_distance;

        let ff = state.distance_front;

        // find the closest obstacle on each side
        let left  = state.distance_front_left;
        let right = state.distance_front_right;

        // determine avoidance action
        if ff < min_d {
            if left < min_d {
                if right < min_d {
                    Some(Action::EmergencyStop)
                } else {
                    Some(Action::AvoidingObstacleToLeft)
                }
            } else if right < min_d {
                if left < min_d {
                    Some(Action::EmergencyStop)
                } else {
                    Some(Action::AvoidingObstacleToRight)
                }
            } else {
                // turn in direction we were navigating to
                match state.turn {
                    Some(n) => {
                        if n < 0.0 {
                            Some(Action::AvoidingObstacleToRight)
                        } else {
                            Some(Action::AvoidingObstacleToLeft)
                        }
                    }
                    None => Some(Action::AvoidingObstacleToLeft),
                }
            }
        } else if left < min_d {
            Some(Action::AvoidingObstacleToLeft)
        } else if right < min_d {
            Some(Action::AvoidingObstacleToRight)
        } else {
            None
        }
    }

    /// replace the shared state ... using a block here to limit the scope of the mutex
    fn update_shared_state(&self, state: &State, nav_state: &Arc<Mutex<Box<State>>>) -> bool {
        let mut x = nav_state.lock().unwrap();
        match x.action {
            Action::Aborted => {
                println!("Aborting navigation");
                return false;
            }
            _ => {}
        };
        *x = Box::new(state.clone());
        true
    }

    fn close_enough(&self, a: &Location, b: &Location) -> bool {
        (a.lat - b.lat).abs() < self.settings.waypoint_accuracy.0
            && (a.lon - b.lon).abs() < self.settings.waypoint_accuracy.1
    }

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

    let c = Color::new(200, 200, 200, 24); // r, g, b, alpha
    let background = Color::new(50, 50, 50, 24); // r, g, b, alpha

    video.fill_rect(top, 20, 600, top + line_height * 5, &background);

    // COLUMN 1
    let mut y = top + line_height;

    // Line 1 - GPS
    video.draw_text(x1,
                    y,
                    match s.loc {
                        None => format!("GPS: N/A"),
                        Some((lat, lon)) => format!("GPS: {:.*}, {:.*}", 6, lat, 6, lon),
                    },
                    &c);
    y += line_height;

    // Line 2 - next waypoint number
    video.draw_text(x1,
                    y,
                    match s.next_waypoint {
                        None => format!("WP ?: N/A"),
                        Some((n, (lat, lon))) => format!("WP {}: {:.*}, {:.*}", n, 6, lat, 6, lon),
                    },
                    &c);
    y += line_height;


    // Line 3 - difference in GPS co-ordinates (are we there yet?)
    video.draw_text(x1,
                    y,
                    if s.loc.is_some() && s.next_waypoint.is_some() {
                        let gps = s.loc.unwrap();
                        let wp = s.next_waypoint.unwrap().1;
                        format!("DIFF: {:.*}, {:.*}", 6, wp.0-gps.0, 6, wp.1-gps.1)
                    } else {
                        format!("DIFF: N/A")
                    },
                    &c);
    y += line_height;

    // Line 4 - compass
    video.draw_text(x1,
                    y,
                    match s.bearing {
                        None => format!("Compass: N/A"),
                        Some(b) => format!("Compass: {:.*}", 1, b),
                    },
                    &c);
    y += line_height;

    // Line 5 - what is bearing for next WP?
    video.draw_text(x1,
                    y,
                    match s.waypoint_bearing {
                        None => format!("WP: N/A"),
                        Some(b) => format!("WP: {:.*}", 1, b),
                    },
                    &c);

    // Line 5 (still) - how much do we need to turn?
    video.draw_text(x1 + 100,
                    y,
                    match s.turn {
                        None => format!("Turn: N/A"),
                        Some(b) => format!("Turn: {:.*}", 1, b),
                    },
                    &c);

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
                    format!("FL={}, FF={}, FR={}",
                            s.distance_front_left,
                            s.distance_front,
                            s.distance_front_right),
                    &c);
    y += line_height;

    // action
    video.draw_text(x2, y, format!("{:?}", s.action), &c);

    // draw raw LIDAR data points
    let lc1 = Color::new(40, 40, 200, 24); // r, g, b, alpha
    let lc2 = Color::new(200, 40, 40, 24); // r, g, b, alpha

    let cx = 320;
    let cy = 240;

    for i in 0..360 {
        if s.lidar[i] < 400 {
            let distance = (s.lidar[i]/2) as f64;
            let (x,y) = to_point(i, 200_f64, cx, cy);
            if i < 90 {
                video.fill_rect(x-1, y-1, 3, 3, &lc1);
            } else {
                video.fill_rect(x-1, y-1, 3, 3, &lc2);
            }
        }
    }

}

fn to_point(angle: usize, distance: f64, x: u32, y: u32) -> (u32,u32) {
    if angle < 90 {
        let (ox, oy) = calc(angle, distance);
        (x+ox, y-oy)
    } else if angle < 180 {
        let (ox, oy) = calc(angle-90, distance);
        (x+ox, y+oy)
    } else if angle < 270 {
        let (ox, oy) = calc(angle-180, distance);
        (x-ox, y+oy)
    } else {
        let (ox, oy) = calc(angle-270, distance);
        (x-ox, y-oy)
    }
}

fn calc(angle: usize, distance: f64) -> (u32,u32) {
    let angle_radians = (angle as f64).to_radians();
    let ox = (distance * angle_radians.cos()) as u32;
    let oy = (distance * angle_radians.sin()) as u32;
    (ox, oy)
}
