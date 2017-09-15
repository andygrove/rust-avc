use std::sync::{Arc, Mutex};

extern crate libsweep;
use self::libsweep::*;

struct Lidar {
    sweep: Arc<Mutex<Sweep>>,
    points: [u32; 360]
}

impl Lidar {

    pub fn new(port: String) -> Self {
        let sweep = Sweep::new(port).unwrap();
        sweep.start_scanning();
        Lidar { sweep: Arc::new(Mutex::new(sweep)), points: [0; 360] }
    }

    pub fn min(&self, start: usize, end: usize) -> u32 {
        0
    }

    pub fn scan(&mut self) {
        match self.sweep.lock().unwrap().scan() {
            Ok(samples) => {

            },
            Err => println!("scan failed")
        }
    }

}


