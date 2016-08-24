extern crate libc;

use self::libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;

#[link(name="opencv_ffi")]
extern {
    fn video_init() -> i32;
    fn video_capture() -> i32;
    fn video_drawtext(x: u32, y: u32, s: *const c_char) -> i32;
    fn video_write() -> i32;
    fn video_close() -> i32;
}

use super::Car;

pub struct Video {
    camera: u8
}

impl Video {

    pub fn new(camera: u8) -> Self {
        Video { camera: camera }
    }

    pub fn init(&self) {
        unsafe { video_init(); };
    }

    // capture a frame and add instrumentation data
    pub fn capture(&self, car: &Car) {
        unsafe {
            video_capture();
            self.drawText(100, 100, "Testing .... 123 ....");
        };
    }

    fn drawText(&self, x: u32, y: u32, s: &str) {
        //unsafe { video_drawtext(x, y, s); }
    }

    pub fn write(&self) {
        unsafe { video_write(); }
    }

    pub fn close(&self) {
        unsafe { video_close() };
    }
}