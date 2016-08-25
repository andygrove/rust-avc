extern crate libc;

use self::libc::c_char;
use std::ffi::CString;

#[link(name="opencv_ffi")]
extern {
    fn video_init(camera: u32) -> i32;
    fn video_capture() -> i32;
    fn video_drawtext(x: u32, y: u32, s: *const c_char) -> i32;
    fn video_write() -> i32;
    fn video_close() -> i32;
}

pub struct Video {
    camera: u32
}

impl Video {

    pub fn new(camera: u32) -> Self {
        Video { camera: camera }
    }

    pub fn init(&self) -> Result<(), i32> {
        match unsafe { video_init(self.camera) } {
            0 => Ok(()),
            s @ _ => Err(s)
        }
    }

    // capture a frame and add instrumentation data
    pub fn capture(&self) {
        unsafe {
            video_capture();
        };
    }

    pub fn draw_text(&self, x: u32, y: u32, s: String) {
        let cs = CString::new(s).unwrap();
        unsafe { video_drawtext(x, y, cs.as_ptr()) };
    }

    pub fn write(&self) {
        unsafe { video_write(); }
    }

    pub fn close(&self) {
        unsafe { video_close() };
    }
}