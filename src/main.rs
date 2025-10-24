#![warn(clippy::pedantic)]
use anyhow::Context as _;
use poise::serenity_prelude as serenity;

mod cmds;
mod db;
mod ea;
mod nano;
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
    ($s: literal) => {
        std::env::var($s).context(format!("{} not found", $s))?
    };
    ($s: literal, $t: ty) => {
        var!($s)
            .parse::<$t>()
            .context(format!("{} not valid {}", $s, stringify!($t)))?
    };
}

#[shuttle_runtime::main]
async fn nanobot() -> Result<service::NanoBot, shuttle_runtime::Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Set Up Tracing Subscriber
    nano::init_tracing_subscriber();

    // Create connection pool to Postgres DB
    let pool = sqlx::PgPool::connect(&var!("DATABASE_URL"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Run SQLx Migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Load token
    let token = var!("DISCORD_TOKEN");

    // Build Axum Router
    let router = axum::Router::new()
        .route(
            "/export",
            axum::routing::get({
                let pool = pool.clone();
                let export_key = var!("EXPORT_KEY");
                |query| routes::export(pool, query, export_key)
            }),
        )
        .route(
            "/import",
            axum::routing::post({
                let pool = pool.clone();
                let import_key = var!("IMPORT_KEY");
                |body| routes::import(pool, body, import_key)
            }),
        )
        .route("/up", axum::routing::get(routes::up))
        .route(
            "/verify",
            axum::routing::post({
                let pool = pool.clone();
                let verify_key = var!("VERIFY_KEY");
                |body| routes::verify(pool, body, verify_key)
            }),
        );

    // Build Poise Instance
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: cmds::all_commands(),
            event_handler: { |c, e, f, d| Box::pin(nano::event_handler(c, e, f, d)) },
            ..Default::default()
        })
        .setup(move |ctx, _, _| {
            Box::pin(async move {
                ctx.set_activity(Some(serenity::ActivityData::custom(
                    "Verifying members since 2023",
                )));
                Ok(Data {
                    au_ch_id: var!("AU_CHANNEL_ID", _),
                    db: pool,
                    ea_key: var!("EA_API_KEY"),
                    ea_url: var!("EA_API_URL"),
                    fresher: var!("FRESHER_ID", _),
                    gaijin: var!("GAIJIN_ID", _),
                    gn_ch_id: var!("GN_CHANNEL_ID", _),
                    member: var!("MEMBER_ID", _),
                    non_member: var!("NON_MEMBER_ID", _),
                    old_member: var!("OLD_MEMBER_ID", _),
                    server: var!("SERVER_ID", _),
                })
            })
        })
        .build();

    // Build Discord struct
    let discord = service::Discord {
        framework,
        token,
        intents: serenity::GatewayIntents::non_privileged(),
    };

    // Return NanoBot
    Ok(service::NanoBot { discord, router })
}
