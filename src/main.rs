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

macro_rules! secret {
    ($s: literal, $ss: ident) => {
        $ss.get($s).context(format!("{} not found", $s))?
    };
    ($s: literal, $ss: ident, $t: ty) => {
        secret!($s, $ss)
            .parse::<$t>()
            .context(format!("{} not valid {}", $s, stringify!($t)))?
    };
}

#[shuttle_runtime::main]
async fn nanobot(
    #[shuttle_runtime::Secrets] secret_store: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> Result<service::NanoBot, shuttle_runtime::Error> {
    // Set Up Tracing Subscriber
    nano::init_tracing_subscriber();

    // Run SQLx Migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Load token
    let token = secret!("DISCORD_TOKEN", secret_store);

    // Build Axum Router
    let router = axum::Router::new()
        .route(
            "/export",
            axum::routing::get({
                let pool = pool.clone();
                let export_key = secret!("EXPORT_KEY", secret_store);
                |query| routes::export(pool, query, export_key)
            }),
        )
        .route(
            "/import",
            axum::routing::post({
                let pool = pool.clone();
                let import_key = secret!("IMPORT_KEY", secret_store);
                |body| routes::import(pool, body, import_key)
            }),
        )
        .route("/up", axum::routing::get(routes::up))
        .route(
            "/verify",
            axum::routing::post({
                let pool = pool.clone();
                let verify_key = secret!("VERIFY_KEY", secret_store);
                move |body| routes::verify(pool, body, verify_key)
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
                    au_ch_id: secret!("AU_CHANNEL_ID", secret_store, _),
                    db: pool,
                    ea_key: secret!("EA_API_KEY", secret_store),
                    ea_url: secret!("EA_API_URL", secret_store),
                    fresher: secret!("FRESHER_ID", secret_store, _),
                    gaijin: secret!("GAIJIN_ID", secret_store, _),
                    gn_ch_id: secret!("GN_CHANNEL_ID", secret_store, _),
                    member: secret!("MEMBER_ID", secret_store, _),
                    non_member: secret!("NON_MEMBER_ID", secret_store, _),
                    old_member: secret!("OLD_MEMBER_ID", secret_store, _),
                    server: secret!("SERVER_ID", secret_store, _),
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
