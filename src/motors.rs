extern crate serial;

use self::serial::prelude::*;
use self::serial::posix::TTYPort;

use std::time::Duration;


#[allow(unused_variables, dead_code)]
pub struct Motors {
    filename: &'static str,
    port: Option<TTYPort>
}

impl Motors {

    pub fn new(f: &'static str) -> Self {
        Motors { filename: f, port: None }
    }

    #[allow(unused_variables, dead_code)]
    pub fn init(&mut self) {

        let mut port = serial::open(self.filename).unwrap();

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud57600).unwrap();
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }).unwrap();

        port.set_timeout(Duration::from_millis(5000)).unwrap();

        self.port = Some(port);

    }

    #[allow(unused_variables)]
    pub fn set_speed(&self, l: i32, r: i32) {
    }

    pub fn stop(&self) {
    }

    pub fn coast(&self) {
    }
}
