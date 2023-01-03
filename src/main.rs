mod miniwm;

use execute::{shell, Execute};
use std::error::Error;

use miniwm::MiniWM;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wm = MiniWM::new()?;

    let config = config::load_config()?;

    wm.init()?;

    for cmd in config.startup {
        let mut command = shell(cmd);
        command.execute()?;
    }

    Ok(wm.run()?)
}
