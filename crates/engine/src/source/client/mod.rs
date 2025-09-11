use std::{error::Error, io::Write, path::PathBuf, time::Duration};

use http::{Method, Request, Uri};
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Bytes};
use log::{error, warn};
use tokio::net::{TcpStream, UnixStream};
use tokio_util::sync::CancellationToken;

use crate::{
    model::node::ModelNodeId,
    source::{Operation, Source},
};

pub struct ClientSource {
    id: ModelNodeId,
    uri: Uri,
    frequency: Duration,
    schema: Option<serde_json::Value>,
    token: CancellationToken,
    content_type: String,
    buffer: Buffer,
}
pub struct Buffer {
    max: usize,
    location: BufferLocation,
}
pub enum BufferLocation {
    Memory,
    Disk(PathBuf),
}
pub enum ClientRequestType {
    Poll(Duration),
    Stream,
}
pub enum SchemaRef {
    Memory(serde_json::Value),
    Disk(PathBuf),
}

impl Source for ClientSource {
    type Error = crate::Error;

    async fn listen<S>(self, sender: S) -> Result<(), Self::Error>
    where
        S: super::SourceHandler,
    {
        let ClientSource {
            id,
            uri,
            frequency,
            schema,
            token,
            content_type,
            buffer,
        } = self;
        let mut interval = tokio::time::interval(frequency);
        let mut buffer = Vec::new();
        let body = Empty::<Bytes>::new();
        while !token.is_cancelled() {
            match fetch(
                &uri,
                &content_type,
                "application/json",
                body.clone(),
                &mut buffer,
            )
            .await
            {
                Ok(_) => match serde_json::from_slice::<serde_json::Value>(&buffer) {
                    Ok(value) => {
                        if schema
                            .as_ref()
                            .is_none_or(|schema| jsonschema::is_valid(&schema, &value))
                        {
                            match serde_json::from_value(value) {
                                Ok(value) => {
                                    sender.send(id, Operation::Set(value)).await;
                                }
                                Err(err) => {
                                    error!("{err}");
                                }
                            }
                        } else {
                            warn!("invalid data provided");
                        }
                    }
                    Err(err) => {
                        error!("{err}");
                    }
                },
                Err(err) => {
                    error!("{err}");
                }
            }
            interval.tick().await;
        }
        Ok(())
    }
}

async fn fetch<B>(
    uri: &Uri,
    content_type: impl AsRef<str>,
    accept: impl AsRef<str>,
    body: B,
    buffer: &mut dyn Write,
) -> Result<(), ClientError>
where
    B: Body + Send + 'static + Unpin,
    B::Data: Send + Sync,
    B::Error: Into<Box<dyn Error + Send + Sync + 'static>>,
{
    let executor = hyper_util::rt::tokio::TokioExecutor::new();
    let mut sender = match uri.scheme_str() {
        Some("unix") => {
            let connector =
                hyper_util::rt::TokioIo::new(UnixStream::connect(uri.to_string()).await?);

            let (sender, conn) = hyper::client::conn::http2::handshake(executor, connector)
                .await
                .map_err(|err| ClientError::Other(err.into()))?;
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
pub enum ClientError {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}
