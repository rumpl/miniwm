use ::xlib::window_system::{Window, WindowSystem, WindowSystemError};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;
use thiserror::Error;
use x11::xlib;

#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("{0}")]
    DisplayNotFound(#[from] WindowSystemError),
}

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
    window_system: WindowSystem,
    workspaces: BTreeMap<u32, Rc<RefCell<Workspace>>>,
    current_workspace: Rc<RefCell<Workspace>>,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        let ws = WindowSystem::new()?;

        let mut workspaces = BTreeMap::new();
        let workspace = Rc::new(RefCell::new(Workspace::new()));
        let current_workspace = Rc::clone(&workspace);
        workspaces.insert(0, workspace);

        Ok(MiniWM {
            window_system: ws,
            workspaces,
            current_workspace,
        })
    }

    pub fn run(&mut self) -> Result<(), MiniWMError> {
        self.window_system
            .get_windows()
            .iter()
            .try_for_each(|w| self.create_window(*w))?;

        loop {
            let event = self.window_system.next_event();

            match event.get_type() {
                xlib::MapRequest => {
                    let event: xlib::XMapRequestEvent = From::from(event);

                    self.create_window(event.window as Window)?;
                }
                xlib::UnmapNotify => {
                    self.remove_window(event)?;
                }
                xlib::KeyPress => {
                    self.handle_keypress(event)?;
                }
                xlib::KeyRelease => {}
                _ => {
                    println!("unknown event {:?}", event);
                }
            }
        }
    }

    fn create_window(&mut self, window: Window) -> Result<(), MiniWMError> {
        self.current_workspace.borrow_mut().add_window(window);

        self.layout()?;

        self.window_system.raise_window(window);
        self.window_system.show_window(window);

        Ok(())
    }

    fn remove_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XUnmapEvent = From::from(event);
        self.current_workspace
            .borrow_mut()
            .remove_window(&event.window as &Window);

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
                        self.window_system.hide_window(*window);
                    });
                self.current_workspace = Rc::clone(workspace);
                self.layout()?;
            } else {
                self.current_workspace
                    .borrow()
                    .windows
                    .iter()
                    .for_each(|window| {
                        self.window_system.hide_window(*window);
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

        let (width, height) = self.window_system.get_screen_size()?;

        let win_width = width as i32 / ws.windows.len() as i32;

        let mut start = 0;
        ws.windows.iter().for_each(|window| {
            self.window_system.move_window(*window, start, 0_i32);
            self.window_system
                .resize_window(*window, win_width as u32, height as u32);
            self.window_system.show_window(*window);
            start += win_width;
        });

        Ok(())
    }
}
