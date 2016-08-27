extern crate chrono;
extern crate navigation;

use super::video::*;
use super::qik::*;
use super::compass::*;
use super::gps::*;
use super::Config;

use chrono::*;
use navigation::*;

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
pub struct State {
    loc: Option<Location>,
    bearing: Option<f32>,
    next_wp: Option<u8>,
    wp_bearing: Option<f32>,
    action: Option<Action>
}

impl State {

    fn new() -> Self {
        State { loc: None, bearing: None, next_wp: None, wp_bearing: None, action: None }
    }

    fn set_action(&mut self, a: Action) {
        println!("Action: {:?}", a);
        self.action = Some(a);

    }

}

/// group all the IO devices in a single strut to make it easier to pass them around
struct IO<'a> {
    gps: &'a GPS,
    imu: &'a Compass,
    qik: Option<&'a Qik>,
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
//                match io.qik {
//                    None => {},
//                    Some(ref mut q) => {
//                        q.coast(Motor::M0);
//                        q.coast(Motor::M1);
//                    }
//                }
            },
            Some(loc) => {
                if close_enough(&loc, &wp) {
                    state.set_action(Action::ReachedWaypoint { waypoint: wp_num });
                    break;
                }

                match io.imu.get() {
                    None => {
                        state.set_action(Action::WaitingForCompass);
//                        match io.qik {
//                            None => {},
//                            Some(q) => {
//                                q.coast(Motor::M0);
//                                q.coast(Motor::M1);
//                            }
//                        }
                    },
                    Some(b) => {
                        let wp_bearing = loc.calc_bearing_to(&wp);
                        let turn_amount = calc_bearing_diff(b, wp_bearing);
//                        match io.qik {
//                            None => {},
//                            Some(q) => {
//                                if turn_amount < 0_f64 {
//                                    q.set_speed(Motor::M0, 100);
//                                    q.set_speed(Motor::M1, 200);
//                                } else {
//                                    q.set_speed(Motor::M0, 200);
//                                    q.set_speed(Motor::M1, 100);
//                                }
//                            }
//                        }
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

fn foo() {
    let video = Video::new(0);
    let start = UTC::now().timestamp();

    video.init(format!("avc-{}.mp4", start)).unwrap();

    //TODO: start thread to do video capture


    video.close();
}

pub fn avc(conf: &Config, enable_motors: bool) {

    let mut state = State::new();

    //TODO: load waypoints from file
    let waypoints: Vec<Location> = vec![
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
        Location::new(39.8617, -104.6731),
    ];

    let gps = GPS::new(conf.gps_device);
    let imu = Compass::new(conf.imu_device);
    let video = Video::new(0);

    let mut io = IO {
        gps: &gps,
        imu: &imu,
//        qik: Some(Motors::new(conf.qik_device)),
        qik: None,
        video: &video
    };

    //TODO: wait for start button

    // navigate to each waypoint in turn
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("Heading for waypoint {} at {:?}", i+1, waypoint);
        navigate_to_waypoint(i+1, &waypoint, &mut io, &mut state);
    }

    state.set_action(Action::Finished);

//    match io.qik {
//        None => {},
//        Some(q) => {
//            q.set_brake(Motor::M0, 127);
//            q.set_brake(Motor::M1, 127);
//        }
//    }

    println!("Finished");
}
