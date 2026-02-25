use tokio::{io::AsyncReadExt, net::TcpListener};

use crate::{
    errors::{Error, Result},
    gate::{limiter::Limiter, relay::TcpRelayService},
    state::AppState,
};

pub struct AppDaemon {
    app_state: AppState,
    connection_limiter: Limiter,
}

impl AppDaemon {
    pub fn new(state: AppState) -> Self {
        let ip_limit = state.config.ip_limit;
        Self {
            app_state: state,
            connection_limiter: Limiter::new(ip_limit),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let listen_sche = self.app_state.config.listen.scheme();
        if listen_sche != "tcp" {
            tracing::error!("Unsupported protocol: {listen_sche}");
        }

        let listen = &self.app_state.config.listen;
        let listen_host = listen.host_str().ok_or(Error::InvalidUrl)?;
        let listen_port = listen.port().ok_or(Error::InvalidUrl)?;
        let listener = TcpListener::bind((listen_host, listen_port))
            .await
            .map_err(Error::IO)?;

        let protocol_detector = guess::ProtocolChainBuilder::new().all_web().build();

        let mut buffer = [0u8; 256];
        loop {
            if let Ok((mut stream, addr)) = listener.accept().await {
                tracing::info!("Accept connection from {addr}");

                if self.connection_limiter.is_limit_enabled()
                    && let Some(_c_limiter) =
                        self.connection_limiter.try_acquire(addr.ip().to_string())
                {
                } else {
                    continue;
                }

                let detection_readed = match stream.read(&mut buffer).await {
                    Ok(ret) => ret,
                    Err(e) => {
                        tracing::error!("{addr} read data fail: {e}");
                        continue;
                    }
                };
                tracing::debug!("Readed {detection_readed} bytes for protocol detection");
                let detect_result = protocol_detector.detect(&buffer);
                tracing::debug!("Detect result: {:?}", detect_result);
                let is_http_service = matches!(detect_result, Ok(p) if p.is_some());

                let dest = if is_http_service {
                    &self.app_state.config.services.http.dest
                } else {
                    &self.app_state.config.services.minecraft.dest
                };
                tracing::debug!("Choose dest: {dest}");

                let relay = match TcpRelayService::new(stream, dest).await {
                    Ok(mut o) => {
                        if o.send_to_to(&buffer[..detection_readed]).await.is_err() {
                            continue;
                        };
                        o
                    }
                    Err(e) => {
                        tracing::error!("Failed to relay service: from {addr} to {dest}: {e}");
                        continue;
                    }
                };

                tracing::info!("Begin relay: {relay}");
                _ = tokio::spawn(async move {
                    let mut relay_service = relay;
                    _ = relay_service.relay().await;
                });
            }
        }
    }
}
