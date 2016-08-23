
pub struct Compass {
    filename: &'static str
}

impl Compass {

    pub fn new(f: &'static str) -> Self {
        Compass { filename: f }
    }

    pub fn get(&self) -> f64 {
        //TODO: get real compass bearing
        349.5
    }

}
