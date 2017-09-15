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

                        // get a lock on the points array
                        let mut points  = points_clone.lock().unwrap();

                        // reset points
                        let max_distance = 1000;
                        for i in 0..360 {
                            points[i] = max_distance;
                        }

                        // this is one complete scan
//                        let mut first = (samples.first().unwrap().angle / 1000) as usize;
//                        let mut last = (samples.last().unwrap().angle / 1000) as usize;
                        //println!("LIDAR: {}..{} degrees has {} samples", first, last, samples.len());

                        // store the points (but only the good ones)
                        for i in 0..samples.len() {
                            let sample = &samples[i];
                            if sample.signal_strength > 100 && sample.distance > 1 {
                                let angle = sample.angle / 1000;
                                if angle >= 0 && angle < 360 {
                                    points[angle as usize] = sample.distance as u32;
                                }
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


