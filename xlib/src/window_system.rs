use core::slice;
use std::{mem::zeroed, ptr::null, slice::from_raw_parts};
use thiserror::Error;
use x11::{xinerama, xlib};

#[derive(Error, Debug)]
pub enum WindowSystemError {
    #[error("display not found")]
    DisplayNotFound,

    #[error("screen not found")]
    ScreenNotFound,
}

pub type Window = u64;

pub struct WindowSystem {
    display: *mut xlib::Display,
}

impl WindowSystem {
    pub fn new() -> Result<Self, WindowSystemError> {
        let display: *mut xlib::Display = unsafe { xlib::XOpenDisplay(null()) };

        if display.is_null() {
            return Err(WindowSystemError::DisplayNotFound);
        }

        unsafe {
            xlib::XSelectInput(
                display,
                xlib::XDefaultRootWindow(display),
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );

            xlib::XGrabKey(
                display,
                xlib::AnyKey,
                xlib::Mod4Mask,
                xlib::XDefaultRootWindow(display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
        }

        Ok(Self { display })
    }

    pub fn get_windows(&self) -> Vec<Window> {
        unsafe {
            xlib::XGrabServer(self.display);
            let root = xlib::XDefaultRootWindow(self.display);
            let mut unused: u64 = 0;
            let mut children: *mut u64 = zeroed();
            let children_ptr: *mut *mut u64 = &mut children;
            let mut num_children: u32 = 0;
            xlib::XQueryTree(
                self.display,
                root,
                &mut unused,
                &mut unused,
                children_ptr,
                &mut num_children,
            );

            let const_children: *const u64 = children as *const u64;

            let windows = from_raw_parts(const_children, num_children as usize)
                .iter()
                .filter(|&&c| c != root)
                .map(|w| *w as Window)
                .collect::<Vec<Window>>();

            xlib::XUngrabServer(self.display);

            return windows;
        }
    }

    pub fn next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        unsafe { xlib::XNextEvent(self.display, &mut event) };

        return event;
    }

    pub fn show_window(&self, window: Window) {
        unsafe { xlib::XMapWindow(self.display, window) };
    }

    pub fn raise_window(&self, window: Window) {
        unsafe { xlib::XRaiseWindow(self.display, window) };
    }

    pub fn hide_window(&self, window: Window) {
        unsafe { xlib::XUnmapWindow(self.display, window) };
    }

    pub fn move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    pub fn resize_window(&self, window: Window, width: u32, height: u32) {
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }

    pub fn get_screen_size(&self) -> Result<(i16, i16), WindowSystemError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = slice::from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0);

            if let Some(screen) = screen {
                Ok((screen.width, screen.height))
            } else {
                Err(WindowSystemError::ScreenNotFound)
            }
        }
    }
}
