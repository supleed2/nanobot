use crate::{Data, Error};

pub(crate) struct NanoBot {
    pub discord: poise::FrameworkBuilder<Data, Error>,
    pub router: axum::Router,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for NanoBot {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        use std::future::IntoFuture;

        let serve = axum::serve(
            shuttle_runtime::tokio::net::TcpListener::bind(addr)
                .await
                .map_err(shuttle_runtime::CustomError::new)?,
            self.router,
        )
        .into_future();

        tokio::select! {
            _ = self.discord.run_autosharded() => {},
            _ = serve => {},
        };

        Ok(())
    }
}
