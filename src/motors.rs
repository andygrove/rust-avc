use qik::*;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Motion {
    Brake(u8),
    Speed(i8),
}

pub struct Motors<'a> {
    qik: &'a mut Qik
}

impl<'a> Motors<'a> {
    pub fn new(qik: &'a mut Qik) -> Self {
        Motors {
            qik: qik,
        }
    }

    pub fn set(&mut self, left: Motion, right: Motion) {
        self._set(Motor::M0, left);
        self._set(Motor::M1, right);
    }

    fn _set(&mut self, m: Motor, n: Motion) {
        match n {
            Motion::Brake(n) => self.qik.set_brake(m, n),
            Motion::Speed(n) => self.qik.set_speed(m, n),
        }
    }
}
