extern crate navigation;
extern crate serial;

use navigation::*;

use std::env;
use std::io;
use std::time::Duration;

use std::io::prelude::*;
use self::serial::prelude::*;

pub struct GPS {
    filename: &'static str,
    port: Box<SerialPort>
}

impl GPS {

    pub fn new(f: &'static str) -> Self {

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

        GPS { filename: f, port: Box::new(port) }
    }

    pub fn get(&self) -> Option<Location> {



        //TODO: get real location
        Some(Location::new(39.8617, -104.6731))
    }

    fn poll(&mut self) {
        println!("Reading...");
        let mut buf = vec![0_u8; 50];
        let foo = self.port.read(&mut buf[..]).unwrap();

    }

}
