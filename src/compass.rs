extern crate serial;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::prelude::*;
use self::serial::prelude::*;

pub struct Bearing {
    value: Option<f32>
}

impl Bearing {

    fn set(&mut self, n: f32) {
        self.value = Some(n);
    }

}

pub struct Compass {
    filename: &'static str,
    bearing: Arc<Mutex<Bearing>>
}

impl Compass {

    pub fn new(f: &'static str) -> Self {
        Compass {
            filename: f,
            bearing: Arc::new(Mutex::new(Bearing { value: None }))
        }
    }

    pub fn start_thread(&self) {

        let f = self.filename.clone();
        let compass_bearing = self.bearing.clone();

        let mut port = serial::open(f).unwrap();

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud57600).unwrap();
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }).unwrap();

        port.set_timeout(Duration::from_millis(5000)).unwrap();

        // start thread to read from serial port
        let _ = thread::spawn(move || {

            let mut buf : Vec<char> = vec![];
            let mut read_buf = vec![0_u8; 128];

            loop {
                let n = port.read(&mut read_buf[..]).unwrap();
                for i in 0..n {
                    let ch = read_buf[i] as char;
                    if ch=='.' || ch.is_numeric() {
                        buf.push(ch);
                    } else if ch == '\n' {
                        let sentence = String::from(&buf[..]);
                        match sentence.parse::<f32>() {
                            Ok(n) => {
                                //println!("bearing: {}", n);
                                compass_bearing.lock().unwrap().set(n);
                            },
                            Err(e) => println!("Failed to parse bearing '{}' due to {:?}", sentence, e)
                        }
                        buf.clear();
                    }
                }

            }
        });
    }

    pub fn get(&self) -> Option<f32> {
        self.bearing.lock().unwrap().value
    }

}
