use std::thread::sleep_ms;
use std::time::Duration;

//extern crate sysfs_gpio;
//
//use sysfs_gpio::{Direction, Pin};

extern crate serial;

use std::io::prelude::*;
use self::serial::prelude::*;
use self::serial::posix::TTYPort;

#[allow(non_camel_case_types)]
pub enum Motor {
    M0,
    M1
}

#[allow(non_camel_case_types)]
pub enum ConfigParam {
    DEVICE_ID, //0,
    PWM_PARAMETER, //1,
    SHUT_DOWN_MOTORS_ON_ERROR, //2,
    SERIAL_TIMEOUT, //3,
    MOTOR_M0_ACCELERATION, //4,
    MOTOR_M1_ACCELERATION, //5,
    MOTOR_M0_BRAKE_DURATION, //6,
    MOTOR_M1_BRAKE_DURATION, //7,
    MOTOR_M0_CURRENT_LIMIT_DIV_2, //8,
    MOTOR_M1_CURRENT_LIMIT_DIV_2, //9,
    MOTOR_M0_CURRENT_LIMIT_RESPONSE, //10,
    MOTOR_M1_CURRENT_LIMIT_RESPONSE, //11,
}

#[allow(non_camel_case_types)]
enum Command {
    GET_FIRMWARE_VERSION,
    GET_ERROR_BYTE,
    GET_CONFIGURATION_PARAMETER,
    SET_CONFIGURATION_PARAMETER,

    MOTOR_M0_FORWARD,
    MOTOR_M0_FORWARD_8_BIT,
    MOTOR_M0_REVERSE,
    MOTOR_M0_REVERSE_8_BIT,
    MOTOR_M1_FORWARD,
    MOTOR_M1_FORWARD_8_BIT,
    MOTOR_M1_REVERSE,
    MOTOR_M1_REVERSE_8_BIT,

    // 2s9v1 only
    MOTOR_M0_COAST,
    MOTOR_M1_COAST,

    // 2s12v10 only
    MOTOR_M0_BRAKE,
    MOTOR_M1_BRAKE,
    GET_MOTOR_M0_CURRENT,
    GET_MOTOR_M1_CURRENT,
    GET_MOTOR_M0_SPEED,
    GET_MOTOR_M1_SPEED,
}

fn get_cmd_byte(cmd: Command) -> u8 {
    use self::Command::*;
    match cmd {
        // commons
        GET_FIRMWARE_VERSION => 0x81,
        GET_ERROR_BYTE => 0x82,
        GET_CONFIGURATION_PARAMETER => 0x83,
        SET_CONFIGURATION_PARAMETER  => 0x84,
        MOTOR_M0_FORWARD => 0x88,
        MOTOR_M0_FORWARD_8_BIT => 0x89,
        MOTOR_M0_REVERSE => 0x8A,
        MOTOR_M0_REVERSE_8_BIT => 0x8B,
        MOTOR_M1_FORWARD => 0x8C,
        MOTOR_M1_FORWARD_8_BIT => 0x8D,
        MOTOR_M1_REVERSE => 0x8E,
        MOTOR_M1_REVERSE_8_BIT => 0x8F,
        // 2s9v1 only
        MOTOR_M0_COAST => 0x86,
        MOTOR_M1_COAST => 0x87,
        // 2s12v10 only
        MOTOR_M0_BRAKE => 0x86,
        MOTOR_M1_BRAKE => 0x87,
        GET_MOTOR_M0_CURRENT => 0x90,
        GET_MOTOR_M1_CURRENT => 0x91,
        GET_MOTOR_M0_SPEED => 0x92,
        GET_MOTOR_M1_SPEED => 0x93,
    }
}


fn get_config_param_byte(p: ConfigParam) -> u8 {
    use self::ConfigParam::*;
    match p {
        DEVICE_ID => 0,
        PWM_PARAMETER => 1,
        SHUT_DOWN_MOTORS_ON_ERROR => 2,
        SERIAL_TIMEOUT => 3,
        MOTOR_M0_ACCELERATION => 4,
        MOTOR_M1_ACCELERATION => 5,
        MOTOR_M0_BRAKE_DURATION => 6,
        MOTOR_M1_BRAKE_DURATION => 7,
        MOTOR_M0_CURRENT_LIMIT_DIV_2 => 8,
        MOTOR_M1_CURRENT_LIMIT_DIV_2 => 9,
        MOTOR_M0_CURRENT_LIMIT_RESPONSE => 10,
        MOTOR_M1_CURRENT_LIMIT_RESPONSE => 11,
    }
}


pub struct Qik {
    device: String,
//    reset_pin: Pin,
    port: TTYPort,
}

impl Qik {

    pub fn new(device: String, reset_pin: u8) -> Self {

        let mut port = serial::open(&device).unwrap();

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600).unwrap();
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }).unwrap();

        port.set_timeout(Duration::from_millis(5000)).unwrap();

        Qik { device: device, /*reset_pin: Pin::new(reset_pin),*/ port: port }
    }

    pub fn init(&mut self) {
//        self.reset_pin.with_exported(|| {
//            self.reset_pin.set_value(0).unwrap();
//            self.reset_pin.set_direction(Direction::Out).unwrap();
//            thread::sleep(Duration::from_millis(1));
//            self.reset_pin.set_direction(Direction::In).unwrap();
//            thread::sleep(Duration::from_millis(10));
//        });

        // begin(speed); //TODO: do we need to set up serial port here?
        self.write_byte(0xAA);
    }

    pub fn get_firmware_version(&mut self) -> u8 {
        let buf: Vec<u8> = vec![ get_cmd_byte(Command::GET_FIRMWARE_VERSION) ];
        self.write(&buf);
        self.read_byte()
    }

    pub fn get_config(&mut self, p: ConfigParam) -> u8 {
        let cmd: Vec<u8> = vec![
            get_cmd_byte(Command::GET_CONFIGURATION_PARAMETER),
            get_config_param_byte(p)
        ];
        self.write(&cmd);
        self.read_byte()
    }

    pub fn set_config(&mut self, p: ConfigParam, v: u8) -> u8 {
        let cmd: Vec<u8> = vec![
            get_cmd_byte(Command::SET_CONFIGURATION_PARAMETER),
            get_config_param_byte(p),
            v,
            0x55,
            0x2A
        ];
        self.write(&cmd);
        self.read_byte()
    }

    pub fn get_error(&mut self) -> u8 {
        let buf: Vec<u8> = vec![ get_cmd_byte(Command::GET_ERROR_BYTE) ];
        self.write(&buf);
        self.read_byte()
    }

    pub fn set_speed(&mut self, m: Motor, speed: i8) {
        if speed >= 0 {
            // forward
            let cmd: Vec<u8> = vec![
                get_cmd_byte(match m {
                    Motor::M0 => Command::MOTOR_M0_FORWARD_8_BIT,
                    Motor::M1 => Command::MOTOR_M1_FORWARD_8_BIT
                }),
                speed as u8
            ];
            self.write(&cmd);
        } else {
            // reverse
            let cmd: Vec<u8> = vec![
                get_cmd_byte(match m {
                    Motor::M0 => Command::MOTOR_M0_REVERSE_8_BIT,
                    Motor::M1 => Command::MOTOR_M1_REVERSE_8_BIT
                }),
                (0-speed) as u8
            ];
            self.write(&cmd);
        }
    }

    /// 2s9v1 only
    pub fn coast(&mut self, m: Motor) {
        let buf: Vec<u8> = vec![ get_cmd_byte(match m {
            Motor::M0 => Command::MOTOR_M0_COAST,
            Motor::M1 => Command::MOTOR_M1_COAST
        })];
        self.write(&buf);
    }

    /// 2s12v10 only
    pub fn set_brake(&mut self, m: Motor, v: u8) {
        assert!(v<128);
        let cmd: Vec<u8> = vec![
            get_cmd_byte(match m {
                Motor::M0 => Command::MOTOR_M0_BRAKE,
                Motor::M1 => Command::MOTOR_M1_BRAKE
            }),
            v
        ];
        self.write(&cmd);
    }

    /// 2s12v10 only
    pub fn get_speed(&mut self, m: Motor) -> u8 {
        self.write_byte(get_cmd_byte(match m {
            Motor::M0 => Command::GET_MOTOR_M0_SPEED,
            Motor::M1 => Command::GET_MOTOR_M1_SPEED
        }));
        self.read_byte()
    }

    /// 2s12v10 only
    pub fn get_current(&mut self, m: Motor) -> u8 {
        let buf: Vec<u8> = vec![ get_cmd_byte(match m {
            Motor::M0 => Command::GET_MOTOR_M0_CURRENT,
            Motor::M1 => Command::GET_MOTOR_M1_CURRENT
        })];
        self.write(&buf);
        self.read_byte()
    }

    /// 2s12v10 only
    pub fn get_current_milliamps(&mut self, m: Motor) -> u32 {
        self.get_current(m) as u32 * 150
    }

    /// writes a single byte to the serial port
    fn write_byte(&mut self, b: u8) {
        let buf: Vec<u8> = vec![ b ];
        self.write(&buf);
    }

    /// writes a byte buffer to the serial port
    fn write(&mut self, buf: &[u8]) {
        self.port.write_all(buf).unwrap();
    }

    /// reads a single bytes from the serial port
    fn read_byte(&mut self) -> u8 {
        let buf = self.read(1);
        buf[0]
    }

    /// reads varible number of bytes from the serial port
    fn read(&mut self, n: usize) -> Vec<u8> {
        let mut buf = vec![0_u8; n];
        self.port.read_exact(buf.as_mut()).unwrap();
        buf
    }

}
