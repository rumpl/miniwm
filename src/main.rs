mod miniwm;

use std::error::Error;

use miniwm::MiniWM;

fn main() -> Result<(), Box<dyn Error>> {
    let display_name = std::env::var("DISPLAY")?;

    let wm = MiniWM::new(&display_name)?;

    wm.init()?;
    wm.run();

    Ok(())
}
