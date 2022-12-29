mod miniwm;

use std::error::Error;

use miniwm::MiniWM;

fn main() -> Result<(), Box<dyn Error>> {
    let wm = MiniWM::new()?;

    wm.init()?;

    Ok(wm.run()?)
}
