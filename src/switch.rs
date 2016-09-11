extern crate sysfs_gpio;

use sysfs_gpio::{Direction, Pin};
use std::env;
use std::thread;
use std::thread::sleep;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use std::time::Duration;

#[derive(Debug)]
pub struct Switch {
  pin: u64,
  pub state: Arc<Mutex<Option<bool>>>
}

impl Switch {
 
    pub fn new(pin: u64) -> Self {
        Switch { pin: pin, state: Arc::new(Mutex::new(None)) }
    }

    pub fn get(&self) -> Option<bool> {
      let s = self.state.lock().unwrap();
      *s
    }

    pub fn start_thread(&self) {  

        let state = self.state.clone();
        let input = Pin::new(self.pin);

        thread::spawn(move || {
        
        // assume pin was exported before this code is called
        input.set_direction(Direction::In).unwrap();

            // loop forever
            loop {
        
                let baseline = match input.get_value() {
                  Ok(n) => n,
                  _ => 123
                };

                let mut value = baseline;
                for i in 0 .. 9 {
                    value = match input.get_value() {
                      Ok(n) => n,
                      _ => 123
                    };
                    if baseline != value {
                        break;
                    }
                    sleep(Duration::from_millis(10));
                }
                if baseline == value {
                    // 0 means low and 255 means high. low means the switch is ON.
                    let mut s = state.lock().unwrap();
                    *s = Some(baseline==0);
                }
                sleep(Duration::from_millis(250));
            }
    });
  }
}
