use std::env;
use std::path::PathBuf;
use std::process::{Child, Command};

use anyhow::{Context, Result};

// Hardcoded path to the Wand executable
const TRAINER_PATH: &str = r"C:\users\steamuser\AppData\Roaming\Wand\Wand.exe";

fn spawn_process(path: &PathBuf) -> Result<Child> {
    Command::new(path)
        .spawn()
        .with_context(|| format!("Failed to spawn: {}", path.display()))
}

fn main() -> Result<()> {
    // Get the game executable path from argv
    let game_path: PathBuf = env::args()
        .nth(1)
        .context("Usage: trainer-monitor <path-to-game.exe>")?
        .into();

    // Spawn both processes
    let mut game = spawn_process(&game_path)?;
    let mut trainer = spawn_process(&PathBuf::from(TRAINER_PATH))?;

    // Monitor both until they exit
    loop {
        let game_done = game.try_wait()?.is_some();
        let trainer_done = trainer.try_wait()?.is_some();

        if game_done && trainer_done {
            break;
        }

        // Small sleep to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
