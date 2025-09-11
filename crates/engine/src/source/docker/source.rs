use std::time::Duration;

use http::Uri;
use rs_docker::Docker;
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpStream, UnixStream},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::model::ModelNodeId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerSourceProvider {
    uri: String,
    filter: DockerSourceFilter,
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DockerSourceFilter {
    Container,
    Image,
    Network,
    Volume,
}

pub struct DockerSource {
    id: ModelNodeId,
    filter: DockerSourceFilter,
}
pub struct BroadcastDockerWatcher<E> {
    token: CancellationToken,
    handle: JoinHandle<Result<(), E>>,
}
impl BroadcastDockerWatcher<crate::Error> {
    pub fn new(addr: &str, rate: Duration, node_id: ModelNodeId) -> Result<Self, std::io::Error> {
        let cancel = CancellationToken::new();
        let cancel_inner = cancel.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(rate);
            while !cancel_inner.is_cancelled() {
                interval.tick().await;
            }

            Ok(())
        });
        Ok(Self {
            token: cancel,
            handle,
        })
    }
}
async fn monitor_events(uri: &Uri) -> Result<(), DockerError> {
    let executor = hyper_util::rt::tokio::TokioExecutor::new();
    let mut sender = match uri.scheme_str() {
        Some("unix") => {
            let connector =
                hyper_util::rt::TokioIo::new(UnixStream::connect(uri.to_string()).await?);

            let (sender, conn) = hyper::client::conn::http2::handshake(executor, connector)
                .await
                .map_err(|err| DockerError::Other(err.into()))?;
            conn.await?;
            sender
        }
        _ => {
            let connector =
                hyper_util::rt::TokioIo::new(TcpStream::connect(uri.to_string()).await?);
            let (sender, conn) = hyper::client::conn::http2::handshake(executor, connector).await?;
            conn.await?;
            sender
        }
    };

    let request = Request::builder()
        .uri(uri)
        .method(Method::GET)
        .header(http::header::CONTENT_TYPE, content_type.as_ref())
        .header(http::header::ACCEPT, accept.as_ref())
        .body(body)?;
    let mut response = sender.send_request(request).await?;
    while let Some(chunk) = response.frame().await {
        if let Some(chunk) = chunk?.data_ref() {
            buffer.write(chunk)?;
        }
    }
    Ok(())
}
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait DockerWatcher {
    fn docker_mut(&mut self) -> &mut BroadcastDockerWatcher;
    fn docker(&self) -> &BroadcastDockerWatcher;
}
