use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use anyhow::{Context, Result};
use tokio::process::Command;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let game_path: PathBuf = env::args()
        .nth(1)
        .context("Usage: monitor <path-to-game.exe>")?
        .into();

    println!("Starting monitor for: {}", game_path.display());

    // 1. Spawn gRPC Server
    let mut rpc_server = spawn_service("rpc-server")?;
    println!("Spawned rpc-server");

    // 2. Spawn Bridge
    let mut bridge = spawn_service("bridge")?;
    println!("Spawned bridge");

    // 3. TODO: Spawn Shim
    println!("TODO: Spawn shim");

    // 4. Start/Monitor Game
    // Assuming we want to launch the game as well
    let mut game_child = Command::new(&game_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to launch game")?;
    println!("Game started: PID {:?}", game_child.id());

    // 5. Monitor Lifecycle
    loop {
        tokio::select! {
            status = game_child.wait() => {
                println!("Game exited: {:?}", status);
                // Shutdown all managed processes
                let _ = rpc_server.kill().await;
                let _ = bridge.kill().await;
                break;
            }
            status = rpc_server.wait() => {
                eprintln!("RPC Server exited unexpectedly: {:?}", status);
                // Restart?
                rpc_server = spawn_service("rpc-server")?;
            }
            status = bridge.wait() => {
                eprintln!("Bridge exited unexpectedly: {:?}", status);
                // Restart?
                bridge = spawn_service("bridge")?;
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                // Heartbeat / Health check
            }
        }
    }

    Ok(())
}

fn spawn_service(name: &str) -> Result<tokio::process::Child> {
    let exe = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    let mut path = env::current_exe()?;
    path.set_file_name(exe);

    Command::new(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context(format!("Failed to spawn {}", name))
}
