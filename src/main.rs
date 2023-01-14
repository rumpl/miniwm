mod miniwm;

use execute::{shell, Execute};
use std::{
    error::Error,
    sync::mpsc::{channel, Sender},
};

use miniwm::MiniWM;

fn run_wm(tx: Sender<()>) -> Result<(), Box<dyn Error>> {
    let mut wm = MiniWM::new().unwrap();

    tx.send(())?;
    Ok(wm.run()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || match run_wm(tx) {
        Ok(()) => {}
        Err(err) => eprint!("{}", err),
    });

    rx.recv()?;

    let config = config::load_config()?;
    for cmd in config.startup {
        std::thread::spawn(|| {
            let mut command = shell(cmd);
            // unwrapping here because we don't really care (yet)
            command.execute().unwrap();
        });
    }

    handle.join().unwrap();
    Ok(())
}
