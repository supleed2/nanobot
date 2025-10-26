#![warn(clippy::pedantic)]

use anyhow::Context as _;
use poise::serenity_prelude::{self as serenity, ClientBuilder, GatewayIntents};
use std::future::IntoFuture as _;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

mod cmds;
mod db;
mod ea;
mod nano;
mod routes;
mod verify;

/// Program data, which is stored and accessible in all command invocations
struct Data {
    au_ch_id: serenity::ChannelId,
    db: sqlx::SqlitePool,
    ea_key: String,
    ea_url: String,
    fresher: serenity::RoleId,
    gaijin: serenity::RoleId,
    gn_ch_id: serenity::ChannelId,
    member: serenity::RoleId,
    non_member: serenity::RoleId,
    old_member: serenity::RoleId,
    server: serenity::GuildId,
}

type ACtx<'a> = poise::ApplicationContext<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Member {
    discord_id: i64,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PendingMember {
    discord_id: i64,
    shortcode: String,
    realname: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ManualMember {
    discord_id: i64,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Gaijin {
    discord_id: i64,
    name: String,
    university: String,
}

macro_rules! var {
    ($var: literal) => {
        std::env::var($var).context(format!("{} not found", $var))?
    };
    ($var: literal, $type: ty) => {
        var!($var)
            .parse::<$type>()
            .context(format!("{} not valid {}", $var, stringify!($type)))?
    };
    ($var: literal, $type: ty, $default: expr) => {
        var!($var).parse::<$type>().unwrap_or($default)
    };
}
pub(crate) use var;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Set Up Tracing Subscriber
    nano::init_tracing_subscriber();

    // Create cancellation tokens
    let token = CancellationToken::new();
    let axum_token = token.clone();

    // Create signal handler
    tokio::spawn(nano::shutdown_handler(token.clone()));

    // Create Axum cancellation signal
    let signal = async move { axum_token.cancelled().await };

    // Connect to SQLite DB and init
    let pool = nano::init_db(&var!("DATABASE_URL")).await?;

    // Bind to all interfaces on port from environment (default to OS selected)
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], var!("PORT", _, 0)));
    tracing::info!("Listening on http://{addr}");

    // Build Axum Router
    let router = routes::router(pool.clone())?;

    // Create Axum server with graceful shutdown
    let listener = TcpListener::bind(addr).await?;
    let server = axum::serve(listener, router).with_graceful_shutdown(signal);

    // Create Discord Bot client
    let mut client = ClientBuilder::new(var!("DISCORD_TOKEN"), GatewayIntents::non_privileged())
        .framework(nano::nanobot(pool)?)
        .await?;

    // Run futures
    tokio::select! {
        err = client.start_autosharded() => tracing::warn!("Discord client quit: {err:?}"),
        err = server.into_future() => tracing::warn!("Axum server quit: {err:?}"),
        () = token.cancelled() => tracing::info!("Shutting down gracefully..."),
    };

    // Delay for cleanup
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    tracing::info!("Shutdown complete");

    Ok(())
}
