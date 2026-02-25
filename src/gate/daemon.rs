use tokio::{io::AsyncReadExt, net::TcpListener};

use crate::{
    errors::{Error, Result},
    gate::relay::TcpRelayService,
    state::AppState,
};

pub struct AppDaemon {
    app_state: AppState,
}

impl AppDaemon {
    pub fn new(state: AppState) -> Self {
        Self { app_state: state }
    }

    pub async fn start(&self) -> Result<()> {
        let listen_sche = self.app_state.config.listen.scheme();
        if listen_sche != "tcp" {
            tracing::error!("Unsupported protocol: {listen_sche}");
        }

        let host = self
            .app_state
            .config
            .listen
            .host_str()
            .ok_or(Error::InvalidUrl)?;
        let port = self
            .app_state
            .config
            .listen
            .port()
            .ok_or(Error::InvalidUrl)?;
        let listener = TcpListener::bind((host, port)).await.map_err(Error::IO)?;

        let protocol_detector = guess::ProtocolChainBuilder::new().all_web().build();

        loop {
            if let Ok((mut stream, addr)) = listener.accept().await {
                let mut buffer = [0u8; 7];
                stream.read_exact(&mut buffer).await.unwrap();
                let is_http_service =
                    matches!(protocol_detector.detect(&buffer), Ok(p) if p.is_some());

                let dest = if is_http_service {
                    &self.app_state.config.services.http.dest
                } else {
                    &self.app_state.config.services.minecraft.dest
                };

                let relay = match TcpRelayService::new(stream, dest).await {
                    Ok(mut o) => {
                        if o.send_to_to(&buffer).await.is_err() {
                            continue;
                        };
                        o
                    }
                    Err(e) => {
                        tracing::error!("Failed to relay http service: from {addr}: {e}");
                        continue;
                    }
                };

                tracing::info!("Begin relay: {relay}");
                _ = tokio::spawn(async {
                    let mut relay_service = relay;
                    relay_service.relay().await.unwrap();
                });
            }
        }
    }
}
