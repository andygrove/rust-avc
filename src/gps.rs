extern crate serial;
use std::io::prelude::*;
use self::serial::prelude::*;

extern crate navigation;
use navigation::*;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::iter::FromIterator;

pub struct GPS {
    filename: &'static str,
    location: Arc<Mutex<Option<Location>>>,
}

impl GPS {
    pub fn new(f: &'static str) -> Self {
        GPS {
            filename: f,
            location: Arc::new(Mutex::new(None)),
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
            })
            .unwrap();

        port.set_timeout(Duration::from_millis(5000)).unwrap();

        // start thread to read from serial port
        let _ = thread::spawn(move || {

            let mut buf: Vec<char> = vec![];
            let mut read_buf = vec![0_u8; 1024];
            let mut last_lat = 0.0;
            let mut last_lon = 0.0;
            loop {
                let n = port.read(&mut read_buf[..]).unwrap();
                for i in 0..n {
                    let ch = read_buf[i] as char;
                    if ch == '\n' {
                        let sentence : String = buf.iter().cloned().collect();
                        // println!("NMEA: {}", sentence);

                        let parts: Vec<&str> = sentence.split(",").collect();

                        match parts[0] {
                            "$GPGLL" => {

                                let lat = parts[1];     // ddmm.mmmm
                                let lat_ns = parts[2];  // N or S
                                let lon = parts[3];     // ddmm.mmmm
                                let lon_ew = parts[4];  // E or W
                                let _ = parts[5];    // hhmmss.sss
                                let av = parts[6];  // A=valid, V=not valid
                                // println!("{} {}, {} {}", lat, lat_ns, lon, lon_ew);

                                if av == "A" {
                                    match Location::parse_nmea(lat, lat_ns, lon, lon_ew) {
                                        Ok(x) => {
                                            if (last_lat-x.lat).abs() > 0.0000001
                                            || (last_lon-x.lon).abs() > 0.0000001 {
                                                let mut loc = gps_location.lock().unwrap();
                                                *loc = Some(Location::new(x.lat, x.lon));
                                                last_lat = x.lat;
                                                last_lon = x.lon;
                                            }
                                        },
                                        Err(e) => println!("Failed to parse GPS: {}", e)
                                    }
                                }
                            }
                            _ => {}
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
        match *loc {
            Some(Location { lat, lon }) => Some(Location::new(lat,lon)),
            None => None
        }
    }
}
