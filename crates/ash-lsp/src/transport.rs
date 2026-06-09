use std::net::SocketAddr;

use tokio::io::{stdin, stdout};
use tokio::net::TcpListener;
use tower_lsp_server::{LspService, Server};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::AshLanguageServer;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("ash_lsp=info,ash_lsp_core=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

pub async fn run_stdio() {
    info!("starting ash-lsp over stdio");
    let (service, socket) = LspService::new(AshLanguageServer::new);
    Server::new(stdin(), stdout(), socket).serve(service).await;
}

pub async fn run_tcp(port: u16) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!(%addr, "starting ash-lsp TCP listener");
    let listener = TcpListener::bind(addr).await?;
    let (stream, peer_addr) = listener.accept().await?;
    info!(%peer_addr, "accepted TCP LSP connection");

    let (read, write) = tokio::io::split(stream);
    let (service, socket) = LspService::new(AshLanguageServer::new);
    Server::new(read, write, socket).serve(service).await;
    Ok(())
}
