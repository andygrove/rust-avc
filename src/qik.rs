extern crate sysfs_gpio;

use sysfs_gpio::{Direction, Pin};
use std::thread::sleep_ms;

enum Command {
    QIK_GET_FIRMWARE_VERSION,
    QIK_GET_ERROR_BYTE,
    QIK_GET_CONFIGURATION_PARAMETER,
    QIK_SET_CONFIGURATION_PARAMETER,
    
    QIK_MOTOR_M0_FORWARD,
    QIK_MOTOR_M0_FORWARD_8_BIT,
    QIK_MOTOR_M0_REVERSE,
    QIK_MOTOR_M0_REVERSE_8_BIT,
    QIK_MOTOR_M1_FORWARD,
    QIK_MOTOR_M1_FORWARD_8_BIT,
    QIK_MOTOR_M1_REVERSE,
    QIK_MOTOR_M1_REVERSE_8_BIT,
    
    // 2s9v1 only
    QIK_2S9V1_MOTOR_M0_COAST,
    QIK_2S9V1_MOTOR_M1_COAST,
    
    // 2s12v10 only
    QIK_2S12V10_MOTOR_M0_BRAKE,
    QIK_2S12V10_MOTOR_M1_BRAKE,
    QIK_2S12V10_GET_MOTOR_M0_CURRENT,
    QIK_2S12V10_GET_MOTOR_M1_CURRENT,
    QIK_2S12V10_GET_MOTOR_M0_SPEED,
    QIK_2S12V10_GET_MOTOR_M1_SPEED,
}

fn get_cmd_byte(cmd: Command) -> u8 {
    match cmd {
        // commons
        QIK_GET_FIRMWARE_VERSION => 0x81,
        QIK_GET_ERROR_BYTE => 0x82,
        QIK_GET_CONFIGURATION_PARAMETER => 0x83,
        QIK_SET_CONFIGURATION_PARAMETER  => 0x84,
        QIK_MOTOR_M0_FORWARD => 0x88,
        QIK_MOTOR_M0_FORWARD_8_BIT => 0x89,
        QIK_MOTOR_M0_REVERSE => 0x8A,
        QIK_MOTOR_M0_REVERSE_8_BIT => 0x8B,
        QIK_MOTOR_M1_FORWARD => 0x8C,
        QIK_MOTOR_M1_FORWARD_8_BIT => 0x8D,
        QIK_MOTOR_M1_REVERSE => 0x8E,
        QIK_MOTOR_M1_REVERSE_8_BIT => 0x8F,
        // 2s9v1 only
        QIK_2S9V1_MOTOR_M0_COAST => 0x86,
        QIK_2S9V1_MOTOR_M1_COAST => 0x87,
        // 2s12v10 only
        QIK_2S12V10_MOTOR_M0_BRAKE => 0x86,
        QIK_2S12V10_MOTOR_M1_BRAKE => 0x87,
        QIK_2S12V10_GET_MOTOR_M0_CURRENT => 0x90,
        QIK_2S12V10_GET_MOTOR_M1_CURRENT => 0x91,
        QIK_2S12V10_GET_MOTOR_M0_SPEED => 0x92,
        QIK_2S12V10_GET_MOTOR_M1_SPEED => 0x93,
    }
}

enum ConfigParam {
    QIK_CONFIG_DEVICE_ID, //0,
    QIK_CONFIG_PWM_PARAMETER, //1,
    QIK_CONFIG_SHUT_DOWN_MOTORS_ON_ERROR, //2,
    QIK_CONFIG_SERIAL_TIMEOUT, //3,
    QIK_CONFIG_MOTOR_M0_ACCELERATION, //4,
    QIK_CONFIG_MOTOR_M1_ACCELERATION, //5,
    QIK_CONFIG_MOTOR_M0_BRAKE_DURATION, //6,
    QIK_CONFIG_MOTOR_M1_BRAKE_DURATION, //7,
    QIK_CONFIG_MOTOR_M0_CURRENT_LIMIT_DIV_2, //8,
    QIK_CONFIG_MOTOR_M1_CURRENT_LIMIT_DIV_2, //9,
    QIK_CONFIG_MOTOR_M0_CURRENT_LIMIT_RESPONSE, //10,
    QIK_CONFIG_MOTOR_M1_CURRENT_LIMIT_RESPONSE, //11,
}

fn foo(p: ConfigParam) -> u8 {
    match p {
        QIK_CONFIG_DEVICE_ID => 0,
        QIK_CONFIG_PWM_PARAMETER => 1,
        QIK_CONFIG_SHUT_DOWN_MOTORS_ON_ERROR => 2,
        QIK_CONFIG_SERIAL_TIMEOUT => 3,
        QIK_CONFIG_MOTOR_M0_ACCELERATION => 4,
        QIK_CONFIG_MOTOR_M1_ACCELERATION => 5,
        QIK_CONFIG_MOTOR_M0_BRAKE_DURATION => 6,
        QIK_CONFIG_MOTOR_M1_BRAKE_DURATION => 7,
        QIK_CONFIG_MOTOR_M0_CURRENT_LIMIT_DIV_2 => 8,
        QIK_CONFIG_MOTOR_M1_CURRENT_LIMIT_DIV_2 => 9,
        QIK_CONFIG_MOTOR_M0_CURRENT_LIMIT_RESPONSE => 10,
        QIK_CONFIG_MOTOR_M1_CURRENT_LIMIT_RESPONSE => 11,
    }
}


struct Qik {
    device: String,
    reset_pin: Pin
}

impl Qik {

    fn new(device: String, reset_pin: u8) {
        Qik { device: device, reset_pin: Pin::new(reset_pin) }
    }

    fn init() {
        /*
        // reset the qik
        digitalWrite(_resetPin, LOW);
        pinMode(_resetPin, OUTPUT); // drive low
        delay(1);
        pinMode(_resetPin, INPUT); // return to high-impedance input (reset is internally pulled up on qik)
        delay(10);

        begin(speed);
        write(0xAA); // allow qik to autodetect baud rate
        */

    }
}

/*
byte cmd[5]; // serial command buffer


char PololuQik::getFirmwareVersion()
{
  listen();
  write(QIK_GET_FIRMWARE_VERSION);
  while (available() < 1);
  return read();
}

byte PololuQik::getErrors()
{
  listen();
  write(QIK_GET_ERROR_BYTE);
  while (available() < 1);
  return read();
}

byte PololuQik::getConfigurationParameter(byte parameter)
{
  listen();
  cmd[0] = QIK_GET_CONFIGURATION_PARAMETER;
  cmd[1] = parameter;
  write(cmd, 2);
  while (available() < 1);
  return read();
}

byte PololuQik::setConfigurationParameter(byte parameter, byte value)
{
  listen();
  cmd[0] = QIK_SET_CONFIGURATION_PARAMETER;
  cmd[1] = parameter;
  cmd[2] = value;
  cmd[3] = 0x55;
  cmd[4] = 0x2A;
  write(cmd, 5);
  while (available() < 1);
  return read();
}

void PololuQik::setM0Speed(int speed)
{
  boolean reverse = 0;

  if (speed < 0)
  {
    speed = -speed; // make speed a positive quantity
    reverse = 1; // preserve the direction
  }

  if (speed > 255)
    speed = 255;

  if (speed > 127)
  {
    // 8-bit mode: actual speed is (speed + 128)
    cmd[0] = reverse ? QIK_MOTOR_M0_REVERSE_8_BIT : QIK_MOTOR_M0_FORWARD_8_BIT;
    cmd[1] = speed - 128;
  }
  else
  {
    cmd[0] = reverse ? QIK_MOTOR_M0_REVERSE : QIK_MOTOR_M0_FORWARD;
    cmd[1] = speed;
  }

  write(cmd, 2);
}

void PololuQik::setM1Speed(int speed)
{
  boolean reverse = 0;

  if (speed < 0)
  {
    speed = -speed; // make speed a positive quantity
    reverse = 1; // preserve the direction
  }

  if (speed > 255)
    speed = 255;

  if (speed > 127)
  {
    // 8-bit mode: actual speed is (speed + 128)
    cmd[0] = reverse ? QIK_MOTOR_M1_REVERSE_8_BIT : QIK_MOTOR_M1_FORWARD_8_BIT;
    cmd[1] = speed - 128;
  }
  else
  {
    cmd[0] = reverse ? QIK_MOTOR_M1_REVERSE : QIK_MOTOR_M1_FORWARD;
    cmd[1] = speed;
  }

  write(cmd, 2);
}

void PololuQik::setSpeeds(int m0Speed, int m1Speed)
{
  setM0Speed(m0Speed);
  setM1Speed(m1Speed);
}

// 2s9v1

void PololuQik2s9v1::setM0Coast()
{
  write(QIK_2S9V1_MOTOR_M0_COAST);
}

void PololuQik2s9v1::setM1Coast()
{
  write(QIK_2S9V1_MOTOR_M1_COAST);
}

void PololuQik2s9v1::setCoasts()
{
  setM0Coast();
  setM1Coast();
}

// 2s12v10

void PololuQik2s12v10::setM0Brake(unsigned char brake)
{
  if (brake > 127)
    brake = 127;

  cmd[0] = QIK_2S12V10_MOTOR_M0_BRAKE;
  cmd[1] = brake;
  write(cmd, 2);
}

void PololuQik2s12v10::setM1Brake(unsigned char brake)
{
  if (brake > 127)
    brake = 127;

  cmd[0] = QIK_2S12V10_MOTOR_M1_BRAKE;
  cmd[1] = brake;
  write(cmd, 2);
}

void PololuQik2s12v10::setBrakes(unsigned char m0Brake, unsigned char m1Brake)
{
  setM0Brake(m0Brake);
  setM1Brake(m1Brake);
}

unsigned char PololuQik2s12v10::getM0Current()
{
  listen();
  write(QIK_2S12V10_GET_MOTOR_M0_CURRENT);
  while (available() < 1);
  return read();
}

unsigned char PololuQik2s12v10::getM1Current()
{
  listen();
  write(QIK_2S12V10_GET_MOTOR_M1_CURRENT);
  while (available() < 1);
  return read();
}

unsigned int PololuQik2s12v10::getM0CurrentMilliamps()
{
  return getM0Current() * 150;
}

unsigned int PololuQik2s12v10::getM1CurrentMilliamps()
{
  return getM1Current() * 150;
}

unsigned char PololuQik2s12v10::getM0Speed()
{
  listen();
  write(QIK_2S12V10_GET_MOTOR_M0_SPEED);
  while (available() < 1);
  return read();
}

unsigned char PololuQik2s12v10::getM1Speed()
{
  listen();
  write(QIK_2S12V10_GET_MOTOR_M1_SPEED);
  while (available() < 1);
  return read();
}