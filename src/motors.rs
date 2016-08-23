

pub struct Motors {
    filename: &'static str
}

impl Motors {

    pub fn new(f: &'static str) -> Self {
        Motors { filename: f }
    }

    pub fn brake_left(&self) {
    }

    pub fn brake_right(&self) {
    }

    pub fn coast_left(&self) {
    }

    pub fn coast_right(&self) {
    }

    pub fn left_speed(&self, speed: i32) {
    }

    pub fn right_speed(&self, speed: i32) {
    }

    pub fn set_speed(&self, l: i32, r: i32) {
    }

    pub fn stop(&self) {
    }
}
