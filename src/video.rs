use super::Car;

pub struct Video {
    camera: u8
}

impl Video {

    pub fn new(camera: u8) -> Self {
        Video { camera: camera }
    }

    pub fn init(&self) {
        //TODO:
    }

    // capture a frame and add instrumentation data
    pub fn capture(&self, car: &Car) {
    }

    pub fn close(&self) {

    }
}