use std::sync::{Arc, Mutex};
use std::thread;

extern crate libsweep;
use self::libsweep::*;

struct Lidar {
    sweep: Arc<Mutex<Sweep>>,
    points: [u32; 360]
}

impl Lidar {

    pub fn new(port: String) -> Self {
        let sweep = Sweep::new(port).unwrap();
        sweep.start_scanning().unwrap();
        Lidar { sweep: Arc::new(Mutex::new(sweep)), points: [0; 360] }
    }

//    fn start_thread(&self) {
//        let lidar = self.sweep.clone();
//        let _ = thread::spawn(move || {
//            loop {
//                lidar.lock().unwrap().scan();
//            }
//        });
//    }

    pub fn min(&self, start: usize, end: usize) -> u32 {
        let mut min = self.points[start];
        for i in start..end {
            if self.points[i] < min {
                min = self.points[i]
            }
        }
        min
    }

    pub fn scan(&mut self) {
        match self.sweep.lock().unwrap().scan() {
            Ok(ref samples) if samples.len() > 0 => {
                let first = (samples.first().unwrap().angle / 100) as usize;
                let last = (samples.first().unwrap().angle / 100) as usize;
                // reset range
                let max_distance = 1000;
                if (first < last) {
                    for i in first..last {
                        self.points[i] = max_distance;
                    }
                } else {
                    // wrap around 360 point
                    for i in first..360 {
                        self.points[i] = max_distance;
                    }
                    for i in 0..last {
                        self.points[i] = max_distance;
                    }
                }
                for i in 0..samples.len() {
                    let sample = &samples[i];
                    let angle = (sample.angle / 100) as usize;
                    self.points[angle] = sample.distance as u32;
                }
            },
            _ => println!("scan failed")
        }
    }

}


