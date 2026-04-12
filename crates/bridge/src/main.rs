use anyhow::{Context, Result};
use tokio::io::{copy_bidirectional, AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use common::handshake::Handshake;

#[cfg(windows)]
use tokio::net::windows::named_pipe::NamedPipeClient;

#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(not(windows))]
struct NamedPipeClient; // Mock

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(unix)]
    {
        let socket_path = "/tmp/trainer-monitor.sock";
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;
        println!("Bridge listening on AF_UNIX: {}", socket_path);

        loop {
            let (mut unix_stream, _) = listener.accept().await?;
            println!("New connection from external client");

            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut unix_stream).await {
                    eprintln!("Bridge error: {:?}", e);
                }
            });
        }
    }

    #[cfg(windows)]
    {
        // Fallback or development TCP listener
        let listener = TcpListener::bind("127.0.0.1:4444").await?;
        println!("Bridge listening on 127.0.0.1:4444");

        loop {
            let (mut tcp_stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut tcp_stream).await {
                    eprintln!("Bridge error: {:?}", e);
                }
            });
        }
    }

    #[cfg(not(any(windows, unix)))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

async fn handle_connection<S>(client_stream: &mut S) -> Result<()> 
where S: AsyncRead + AsyncWrite + Unpin {
    #[cfg(windows)]
    {
        let mut pipe_client = NamedPipeClient::connect(r"\\.\pipe\deckcheatz")
            .context("Failed to connect to monitor named pipe")?;

        // 1. Initiate Handshake with RPC Server
        let handshake = Handshake::new();
        handshake.send(&mut pipe_client).await.context("Handshake send failed")?;
        
        let response = Handshake::receive(&mut pipe_client).await.context("Handshake receive failed")?;
        println!("Handshake with RPC Server successful: {:?}", response);

        // 2. Continuous Proxy Loop
        copy_bidirectional(client_stream, &mut pipe_client).await?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        // On Linux, the Bridge itself might be running in a context where it needs to 
        // reach the Windows Named Pipe (e.g., via a proxy or shared mount if applicable),
        // but typically the Bridge is on the Windows/Wine side.
        // If we are on Linux, we might be the *other* side of the AF_UNIX.
        Err(anyhow::anyhow!("Bridge transport to Named Pipe requires Windows/Wine"))
    }
}
