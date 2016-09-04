extern crate libc;

use self::libc::c_char;
use std::ffi::CString;

#[link(name="opencv_ffi")]
extern {
    fn video_init(camera: u32, filename: *const c_char) -> i32;
    fn video_capture() -> i32;
    fn video_drawtext(x: u32, y: u32, s: *const c_char, r: u8, g: u8, b: u8, a: u8) -> i32;
    fn video_write() -> i32;
    fn video_close() -> i32;
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

impl Color {

    pub fn new(r: u8, g: u8, b: u8, a: u8) {
        Color { r: r, g: g, b: b, a: a }
    }

}

pub struct Video {
    camera: u32
}

impl Video {

    pub fn new(camera: u32) -> Self {
        Video { camera: camera }
    }

    pub fn init(&self, filename: String) -> Result<(), i32> {
        let f = CString::new(filename).unwrap();
        match unsafe { video_init(self.camera, f.as_ptr()) } {
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

    pub fn draw_text(&self, x: u32, y: u32, s: String, c: &Color) {
        let cs = CString::new(s).unwrap();
        unsafe { video_drawtext(x, y, cs.as_ptr(), c.r, c.g, c.b, c.a) };
    }

    pub fn write(&self) {
        unsafe { video_write(); }
    }

    pub fn close(&self) {
        unsafe { video_close() };
    }
}