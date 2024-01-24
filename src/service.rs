use crate::{Data, Error};
use poise::serenity_prelude as serenity;

pub(crate) struct NanoBot {
    pub discord: Discord,
    pub router: axum::Router,
}

pub(crate) struct Discord {
    pub framework: poise::Framework<Data, Error>,
    pub token: String,
    pub intents: serenity::GatewayIntents,
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

        let mut client = serenity::ClientBuilder::new(self.discord.token, self.discord.intents)
            .framework(self.discord.framework)
            .await
            .map_err(shuttle_runtime::CustomError::new)?;

        tokio::select! {
            _ = client.start_autosharded() => {},
            _ = serve => {},
        };

        Ok(())
    }
}
