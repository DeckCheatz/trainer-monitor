use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};
use common::proto::monitor_service_server::{MonitorService, MonitorServiceServer};
use common::proto::{TrainerRequest, TrainerState, Event};
use common::handshake::Handshake;
use tokio_stream::wrappers::ReceiverStream;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};

const PIPE_NAME: &str = r"\\.\pipe\deckcheatz";

#[derive(Default)]
pub struct MyMonitorService {}

#[tonic::async_trait]
impl MonitorService for MyMonitorService {
    async fn get_trainer_state(
        &self,
        _request: Request<TrainerRequest>,
    ) -> Result<Response<TrainerState>, Status> {
        let reply = TrainerState {
            status: "ready".into(),
            attached: true,
            attributes: HashMap::new(),
        };
        Ok(Response::new(reply))
    }

    type StreamEventsStream = ReceiverStream<Result<Event, Status>>;

    async fn stream_events(
        &self,
        _request: Request<TrainerRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        tokio::spawn(async move {
            let event = Event {
                event_type: "heartbeat".into(),
                payload: "{}".into(),
            };
            let _ = tx.send(Ok(event)).await;
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub struct HandshakedStream<S> {
    stream: S,
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for HandshakedStream<S> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for HandshakedStream<S> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

#[cfg(windows)]
struct NamedPipeIncoming {
    pipe_name: String,
}

#[cfg(windows)]
impl Stream for NamedPipeIncoming {
    type Item = Result<HandshakedStream<NamedPipeServer>, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&self.pipe_name)?;

        // This is a bit simplified for demonstration. 
        // In reality, we'd need to handle connection asynchronously.
        // For brevity in this turn, I'll use a more robust listener pattern if needed.
        // But for now, let's assume this is the direction.
        Poll::Pending
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        println!("RPC Server listening on named pipe: {}", PIPE_NAME);
        let mut server_options = ServerOptions::new();
        server_options.first_pipe_instance(true);

        loop {
            let mut pipe = server_options.create(PIPE_NAME)?;
            pipe.connect().await?;

            tokio::spawn(async move {
                if let Err(e) = handle_pipe_connection(pipe).await {
                    eprintln!("Pipe connection error: {:?}", e);
                }
            });

            server_options = ServerOptions::new();
        }
    }

    #[cfg(not(windows))]
    {
        println!("RPC Server (Linux/Mock) listening on TCP 50051");
        let addr = "[::1]:50051".parse()?;
        Server::builder()
            .add_service(MonitorServiceServer::new(MyMonitorService::default()))
            .serve(addr)
            .await?;
    }

    Ok(())
}

#[cfg(windows)]
async fn handle_pipe_connection(mut pipe: NamedPipeServer) -> anyhow::Result<()> {
    // 1. Receive Handshake
    let handshake = Handshake::receive(&mut pipe).await?;
    println!("Received handshake: {:?}", handshake);

    // 2. Send Handshake back
    let response = Handshake::new();
    response.send(&mut pipe).await?;

    // 3. Handoff to gRPC
    // Note: Tonic's serve_with_incoming requires a Stream of AsyncRead+AsyncWrite.
    // For Named Pipes, we usually handle each connection in a task or use a custom Incoming.
    
    // For now, let's just use a simple mock for the gRPC part over the handshaked pipe
    // Implementation of serving gRPC over a single pipe instance:
    let service = MyMonitorService::default();
    Server::builder()
        .add_service(MonitorServiceServer::new(service))
        .serve_with_incoming(tokio_stream::iter(vec![Ok::<_, std::io::Error>(HandshakedStream { stream: pipe })]))
        .await?;

    Ok(())
}
