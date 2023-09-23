use crate::{Data, Error};

pub(crate) struct NanoBot {
    pub discord: poise::FrameworkBuilder<Data, Error>,
    pub router: axum::Router,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for NanoBot {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let serve = axum::Server::bind(&addr).serve(self.router.into_make_service());

        tokio::select! {
            _ = self.discord.run_autosharded() => {},
            _ = serve => {},
        };

        Ok(())
    }
}
