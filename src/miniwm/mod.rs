use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem::zeroed;
use std::ptr::null;
use std::rc::Rc;
use std::{collections::BTreeSet, slice};

use thiserror::Error;
use x11::{xinerama, xlib};

#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("display not found")]
    DisplayNotFound,

    #[error("screen not found")]
    ScreenNotFound,
}

pub type Window = u64;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
struct Workspace {
    windows: BTreeSet<Window>,
}

impl Workspace {
    fn new() -> Self {
        Workspace {
            windows: BTreeSet::new(),
        }
    }

    fn add_window(&mut self, window: Window) {
        self.windows.insert(window);
    }

    fn remove_window(&mut self, window: &Window) {
        self.windows.remove(window);
    }
}

pub struct MiniWM {
    display: *mut xlib::Display,
    windows: BTreeSet<Window>,
    workspaces: BTreeMap<u32, Rc<RefCell<Workspace>>>,
    current_workspace: Rc<RefCell<Workspace>>,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        let display: *mut xlib::Display = unsafe { xlib::XOpenDisplay(null()) };

        if display.is_null() {
            return Err(MiniWMError::DisplayNotFound);
        }

        let mut workspaces = BTreeMap::new();
        let workspace = Rc::new(RefCell::new(Workspace::new()));
        let current_workspace = Rc::clone(&workspace);
        workspaces.insert(0, workspace);

        Ok(MiniWM {
            display,
            windows: BTreeSet::new(),
            workspaces,
            current_workspace,
        })
    }

    pub fn init(&self) -> Result<(), MiniWMError> {
        unsafe {
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );

            xlib::XGrabKey(
                self.display,
                xlib::AnyKey,
                xlib::Mod4Mask,
                xlib::XDefaultRootWindow(self.display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
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
                    xlib::KeyPress => {
                        self.handle_keypress(event)?;
                    }
                    xlib::KeyRelease => {}
                    _ => {
                        // println!("unknown event {:?}", event);
                    }
                }
            }
        }
    }

    fn create_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XMapRequestEvent = From::from(event);

        self.current_workspace
            .borrow_mut()
            .add_window(event.window as Window);
        self.windows.insert(event.window as Window);
        self.layout()?;
        unsafe { xlib::XMapRaised(self.display, event.window) };

        Ok(())
    }

    fn remove_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XUnmapEvent = From::from(event);
        self.current_workspace
            .borrow_mut()
            .remove_window(&event.window as &Window);
        self.windows.remove(&event.window);

        self.layout()
    }

    fn handle_keypress(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XKeyEvent = From::from(event);

        if let 10..=19 = event.keycode {
            let ws = event.keycode - 10;
            if let Some(workspace) = self.workspaces.get(&ws) {
                self.current_workspace
                    .borrow()
                    .windows
                    .iter()
                    .for_each(|window| {
                        self.hide_window(*window);
                    });
                self.current_workspace = Rc::clone(workspace);
                self.layout()?;
            } else {
                self.current_workspace
                    .borrow()
                    .windows
                    .iter()
                    .for_each(|window| {
                        self.hide_window(*window);
                    });
                let workspace = Rc::new(RefCell::new(Workspace::new()));
                self.current_workspace = Rc::clone(&workspace);
                self.workspaces.insert(ws, workspace);
                self.layout()?;
            }

            self.layout()?;
            println!("got control+{}", event.keycode);
        }
        Ok(())
    }

    fn layout(&mut self) -> Result<(), MiniWMError> {
        let ws = self.current_workspace.borrow();
        if ws.windows.is_empty() {
            return Ok(());
        }

        let (width, height) = self.get_screen_size()?;

        let win_width = width as i32 / ws.windows.len() as i32;

        let mut start = 0;
        ws.windows.iter().for_each(|window| {
            self.move_window(*window, start, 0_i32);
            self.resize_window(*window, win_width as u32, height as u32);
            self.show_window(*window);
            start += win_width;
        });

        Ok(())
    }

    fn show_window(&self, window: Window) {
        unsafe { xlib::XMapWindow(self.display, window) };
    }

    fn hide_window(&self, window: Window) {
        unsafe { xlib::XUnmapWindow(self.display, window) };
    }

    fn move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    fn resize_window(&self, window: Window, width: u32, height: u32) {
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }

    fn get_screen_size(&self) -> Result<(i16, i16), MiniWMError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = slice::from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0);

            if let Some(screen) = screen {
                Ok((screen.width, screen.height))
            } else {
                Err(MiniWMError::ScreenNotFound)
            }
        }
    }
}
