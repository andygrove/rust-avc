extern crate navigation;

use navigation::*;


pub struct GPS {
    filename: &'static str
}

impl GPS {

    pub fn new(f: &'static str) -> Self {
        GPS { filename: f }
    }

    pub fn get(&self) -> Option<Location> {
        //TODO: get real location
        Some(Location::new(39.8617, -104.6731))
    }
}
