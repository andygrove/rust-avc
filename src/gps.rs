extern crate serial;

extern crate navigation;

use navigation::*;

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
        let _ = thread::spawn(move || {

            let mut buf : Vec<char> = vec![];
            let mut read_buf = vec![0_u8; 128];

            loop {
                let n = port.read(&mut read_buf[..]).unwrap();
                for i in 0..n {
                    let ch = read_buf[i] as char;
                    if ch=='\n' {
                        let sentence = String::from(&buf[..]);
                        //println!("NMEA: {}", sentence);

                        let parts: Vec<&str> = sentence.split(",").collect();

                        match parts[0] {
                            "$GPGLL" => {

                                let lat = parts[1];     // ddmm.mmmm
                                let lat_ns = parts[2];  // N or S
                                let lon = parts[3];     // ddmm.mmmm
                                let lon_ew = parts[4];  // E or W
                                let _ = parts[5];    // hhmmss.sss
                                let _ = parts[6];  // A=valid, V=not valid

                                println!("{} {}, {} {}", lat, lat_ns, lon, lon_ew);

                                let x = Location::parse_nmea(lat, lat_ns, lon, lon_ew);

                                //TODO: only update the shared state if the co-ords changed

                                // on receive valid co-ords ...
                                let mut loc = gps_location.lock().unwrap();
                                loc.set(x.lat, x.lon);

                            },
                            _ => {

                            }
                        }

                        buf.clear();
                    } else {
                        buf.push(ch);
                    }
                }



            }
        });
    }

    pub fn get(&self) -> Option<Location> {
        let loc = self.location.lock().unwrap();
        Some(Location { lat: loc.lat, lon: loc.lon })
    }

}
