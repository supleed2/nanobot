use anyhow::Context as _;
use poise::serenity_prelude as serenity;

mod cmds;
mod db;
mod ea;
mod routes;
mod service;
mod verify;

/// Program data, which is stored and accessible in all command invocations
struct Data {
    au_ch_id: serenity::ChannelId,
    db: sqlx::PgPool,
    ea_key: String,
    ea_url: String,
    fresher: serenity::RoleId,
    member: serenity::RoleId,
}

type ACtx<'a> = poise::ApplicationContext<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct Member {
    discord_id: i64,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
}

#[derive(Debug)]
struct PendingMember {
    discord_id: i64,
    shortcode: String,
    realname: String,
}

#[derive(Debug)]
struct ManualMember {
    discord_id: i64,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
}

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
    // #[shuttle_shared_db::Postgres] _pool: sqlx::PgPool,
) -> shuttle_poise::ShuttlePoise<Data, Error> {
    Ok(poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), setup()],
            ..Default::default()
        })
        .token(
            secret_store
                .get("DISCORD_TOKEN")
                .context("DISCORD_TOKEN not found")?,
        )
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|_, _, _| Box::pin(async { Ok(Data {}) }))
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?
        .into())
}

// Links for info:
// https://github.com/serenity-rs/poise
// https://github.com/serenity-rs/poise/tree/current/examples
// https://github.com/shuttle-hq/shuttle-examples/blob/main/poise/hello-world/src/main.rs
// https://github.com/rust-community-discord/ferrisbot-for-discord/blob/main/src/main.rs#L154
