use std::sync::{Arc, Mutex};
use std::thread;

extern crate libsweep;
use self::libsweep::*;

pub struct Lidar {
    points: Arc<Mutex<[u32; 360]>>
}

impl Lidar {

    pub fn new(port: String) -> Self {

        let lidar = Lidar { points: Arc::new(Mutex::new([0; 360])) };

        let points_clone = lidar.points.clone();

        let _ = thread::spawn(move || {
            let sweep = Sweep::new(port).unwrap();
            sweep.start_scanning().unwrap();
            loop {
                match sweep.scan() {
                    Ok(ref samples) if samples.len() > 0 => {
                        let mut first = (samples.first().unwrap().angle / 100) as usize;
                        let mut last = (samples.first().unwrap().angle / 100) as usize;
                        if first > 359 { first = 359 };
                        if last > 359 { last = 359 };

                        let mut points  = points_clone.lock().unwrap();

                        // reset range
                        let max_distance = 1000;
                        if (first < last) {
                            for i in first..last {
                                points[i] = max_distance;
                            }
                        } else {
                            // wrap around 360 point
                            for i in first..359 {
                                points[i] = max_distance;
                            }
                            for i in 0..last {
                                points[i] = max_distance;
                            }
                        }
                        for i in 0..samples.len() {
                            let sample = &samples[i];
                            let angle = (sample.angle / 100) as usize;
                            if angle > 1 && angle < 360 {
                                points[angle] = sample.distance as u32;
                            }
                        }

                    },
                    _ => println!("scan failed")
                }
            }
        });

        lidar
    }

    pub fn min(&self, start: usize, end: usize) -> u32 {
        let points  = self.points.lock().unwrap();
        let mut min = points[start];
        if start < end {
            for i in start..end {
                if points[i] < min {
                    min = points[i]
                }
            }

        } else {
            for i in start..360 {
                if points[i] < min {
                    min = points[i]
                }
            }
            for i in 0..end {
                if points[i] < min {
                    min = points[i]
                }
            }

        }
        min
    }

}


