extern crate navigation;
extern crate serial;

use navigation::*;

use std::env;
use std::io;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::prelude::*;
use self::serial::prelude::*;

pub struct GPS {
    filename: &'static str,
    location: Arc<Mutex<Location>>
}

impl GPS {

    pub fn new(f: &'static str) -> Self {
        GPS {
            filename: f,
            location: Arc::new(Mutex::new(Location::new(0 as f64, 0 as f64)))
        }
    }

    pub fn start_thread(&self) {

        let f = self.filename.clone();
        let gps_location = self.location.clone();

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
        let handle = thread::spawn(move || {

            let mut buf = vec![0_u8; 128];
            let mut read_buf = vec![0_u8; 128];

            loop {
                println!("Reading from GPS...");
                let n = port.read(&mut read_buf[..]).unwrap();

                println!("Read {} bytes from GPS...", n);
                buf.extend_from_slice(&read_buf[0..n]);

                //TODO: parse NMEA sentence

                //TODO: only update location if GPS has changed

                // on receive valid co-ords ...
                let mut loc = gps_location.lock().unwrap();
                loc.set(12.3 as f64, 45.6 as f64);

            }
        });
    }

    pub fn get(&self) -> Option<Location> {
        let loc = self.location.lock().unwrap();
        Some(Location { lat: loc.lat, lon: loc.lon })
    }

}
