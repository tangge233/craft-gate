use std::fmt::Display;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;
use uuid::Uuid;

use crate::errors::{Error, Result};

pub struct TcpRelayService {
    from: TcpStream,
    to: TcpStream,
    id: String,
}

impl TcpRelayService {
    pub async fn new(from: TcpStream, to: &Url) -> Result<Self> {
        if to.scheme() != "tcp" {
            return Err(Error::InvalidUrl);
        }

        let host = to.host_str().ok_or(Error::InvalidUrl)?;
        let port = to.port().ok_or(Error::InvalidUrl)?;
        let to_stream = TcpStream::connect((host, port)).await.map_err(Error::IO)?;

        Ok(TcpRelayService {
            from,
            to: to_stream,
            id: Uuid::new_v4().to_string(),
        })
    }

    pub async fn relay(&mut self) -> Result<()> {
        let (mut from_read, mut from_write) = self.from.split();
        let (mut to_read, mut to_write) = self.to.split();

        // from -> to
        let forward_fut = async {
            match io::copy(&mut from_read, &mut to_write).await {
                Ok(bytes) => {
                    tracing::debug!("Forwarded {} bytes", bytes);
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Forward error: {}", e);
                    Err(Error::IO(e))
                }
            }
        };

        // to -> from
        let backward_fut = async {
            match io::copy(&mut to_read, &mut from_write).await {
                Ok(bytes) => {
                    tracing::debug!("Backward forwarded {} bytes", bytes);
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Backward error: {}", e);
                    Err(Error::IO(e))
                }
            }
        };

        tokio::select! {
            result = forward_fut => {
                tracing::info!("Forward direction completed");
                result
            }
            result = backward_fut => {
                tracing::info!("Backward direction completed");
                result
            }
        }
    }

    /// 向 from 流发送数据
    pub async fn send_to_from(&mut self, data: &[u8]) -> Result<()> {
        self.from.write_all(data).await.map_err(Error::IO)?;
        tracing::debug!("Sent {} bytes to from stream", data.len());
        Ok(())
    }

    /// 向 to 流发送数据
    pub async fn send_to_to(&mut self, data: &[u8]) -> Result<()> {
        self.to.write_all(data).await.map_err(Error::IO)?;
        tracing::debug!("Sent {} bytes to to stream", data.len());
        Ok(())
    }

    /// 从 from 流读取数据
    pub async fn read_from_from(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.from.read(buf).await.map_err(Error::IO)?;
        tracing::debug!("Read {} bytes from from stream", n);
        Ok(n)
    }

    /// 从 to 流读取数据
    pub async fn read_from_to(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.to.read(buf).await.map_err(Error::IO)?;
        tracing::debug!("Read {} bytes from to stream", n);
        Ok(n)
    }
}

impl Display for TcpRelayService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from_addr = match self.from.local_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "unknown".to_string(),
        };
        let to_addr = match self.to.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "unknown".to_string(),
        };

        write!(
            f,
            "TcpRelay[id={}, from={}, to={}]",
            self.id, from_addr, to_addr
        )
    }
}
