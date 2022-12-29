use std::ptr::null;
use std::slice::from_raw_parts;
use std::{ffi::NulError, mem::zeroed};

use x11::{xinerama, xlib};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("display not found")]
    DisplayNotFound,

    #[error("screen not found")]
    ScreenNotFound,

    #[error("{0}")]
    NulString(#[from] NulError),
}

pub struct MiniWM {
    display: *mut xlib::Display,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        let display: *mut xlib::Display = unsafe { xlib::XOpenDisplay(null()) };

        if display.is_null() {
            return Err(MiniWMError::DisplayNotFound);
        }

        Ok(MiniWM { display })
    }

    pub fn init(&self) -> Result<(), MiniWMError> {
        unsafe {
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask,
            );
        }

        Ok(())
    }

    pub fn run(&self) -> Result<(), MiniWMError> {
        println!("miniwm running");
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);

                match event.get_type() {
                    xlib::MapRequest => {
                        self.create_window(event)?;
                    }
                    _ => {
                        println!("unknown event {:?}", event);
                    }
                }
            }
        }
    }

    fn create_window(&self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        self.set_window_fullscreen(event.window)?;
        unsafe { xlib::XMapRaised(self.display, event.window) };
        Ok(())
    }

    fn set_window_fullscreen(&self, window: u64) -> Result<(), MiniWMError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0);

            if let Some(screen) = screen {
                xlib::XResizeWindow(
                    self.display,
                    window,
                    screen.width as u32,
                    screen.height as u32,
                );
            } else {
                return Err(MiniWMError::ScreenNotFound);
            }
        };
        Ok(())
    }
}
