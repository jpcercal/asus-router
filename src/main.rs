mod cmd;
mod resolver;
use cmd::app::boot;

use anyhow::Result;
use env_logger::init;

fn main() -> Result<()> {
    // Initialize the env_logger to set up logging.
    init();

    // Runs the application
    boot()
}
