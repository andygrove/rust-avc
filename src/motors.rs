
#[allow(unused_variables, dead_code)]
pub struct Motors {
    filename: &'static str
}

impl Motors {

    pub fn new(f: &'static str) -> Self {
        Motors { filename: f }
    }

    #[allow(unused_variables)]
    pub fn set_speed(&self, l: i32, r: i32) {
    }

    pub fn stop(&self) {
    }

    pub fn coast(&self) {
    }
}
