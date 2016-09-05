use qik::*;

pub enum Motion {
    Coast,
    Brake(u8),
    Speed(i8)
}

pub struct Motors<'a> {
    qik: &'a mut Qik,
    enabled: bool
}

impl<'a> Motors<'a> {

    pub fn new(qik: &'a mut Qik, enabled: bool) -> Self {
        Motors { qik: qik, enabled: enabled }
    }

    pub fn set(&mut self, left: Motion, right: Motion) {
        self._set(Motor::M0, left);
        self._set(Motor::M1, right);
    }

    fn _set(&mut self, m: Motor, n: Motion) {
        match n {
            Motion::Coast => self.qik.coast(m),
            Motion::Brake(n) => self.qik.set_brake(m, n),
            Motion::Speed(n) => self.qik.set_speed(m, n)
        }
    }
}