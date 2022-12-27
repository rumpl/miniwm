use std::{error::Error, ffi::CString, mem::zeroed};

use x11::xlib;

fn main() -> Result<(), Box<dyn Error>> {
    let display_name = std::env::var("DISPLAY")?;

    let display: *mut xlib::Display =
        unsafe { xlib::XOpenDisplay(CString::new(display_name)?.as_ptr()) };

    if display.is_null() {
        std::process::exit(1);
    }

    unsafe {
        xlib::XSelectInput(
            display,
            xlib::XDefaultRootWindow(display),
            xlib::SubstructureNotifyMask | xlib::SubstructureNotifyMask,
        );
    }

    let mut event: xlib::XEvent = unsafe { zeroed() };
    loop {
        unsafe {
            xlib::XNextEvent(display, &mut event);

            match event.get_type() {
                xlib::MapRequest => {
                    let event: xlib::XMapRequestEvent = From::from(event);
                    xlib::XRaiseWindow(display, event.window);
                }
                _ => {}
            }
        }
    }
}
