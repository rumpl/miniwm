use std::collections::BTreeMap;
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

pub struct Rect(pub f32, pub f32, pub f32, pub f32);

pub type Window = u64;

pub struct MiniWM {
    display: *mut xlib::Display,
    windows: BTreeMap<Window, Rect>,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        let display: *mut xlib::Display = unsafe { xlib::XOpenDisplay(null()) };

        if display.is_null() {
            return Err(MiniWMError::DisplayNotFound);
        }

        Ok(MiniWM {
            display,
            windows: BTreeMap::new(),
        })
    }

    pub fn init(&self) -> Result<(), MiniWMError> {
        unsafe {
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), MiniWMError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);

                match event.get_type() {
                    xlib::MapRequest => {
                        self.create_window(event)?;
                    }
                    xlib::UnmapNotify => {
                        self.remove_window(event)?;
                    }
                    _ => {
                        println!("unknown event {:?}", event);
                    }
                }
            }
        }
    }

    fn remove_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XUnmapEvent = From::from(event);
        self.windows.remove(&event.window);
        self.layout()
    }

    fn create_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        self.windows
            .insert(event.window as Window, Rect(1.0, 1.0, 1.0, 1.0));
        self.layout()?;
        unsafe { xlib::XMapRaised(self.display, event.window) };

        Ok(())
    }

    fn layout(&mut self) -> Result<(), MiniWMError> {
        if self.windows.is_empty() {
            return Ok(());
        }

        let (width, height) = self.get_screen_size()?;

        let win_width = width as usize / self.windows.len();

        let mut start = 0;
        self.windows.iter().for_each(|(window, _)| {
            self.move_window(*window, start, 0_u32);
            self.resize_window(*window, win_width as u32, height as u32);
            start += win_width as u32;
        });

        Ok(())
    }

    fn move_window(&self, window: Window, x: u32, y: u32) {
        unsafe { xlib::XMoveWindow(self.display, window, x as i32, y as i32) };
    }

    fn resize_window(&self, window: Window, width: u32, height: u32) {
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }

    fn get_screen_size(&self) -> Result<(i16, i16), MiniWMError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0);

            if let Some(screen) = screen {
                Ok((screen.width, screen.height))
            } else {
                Err(MiniWMError::ScreenNotFound)
            }
        }
    }
}
