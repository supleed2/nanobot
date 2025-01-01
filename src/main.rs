#![warn(clippy::pedantic)]
use anyhow::Context as _;
use poise::serenity_prelude::{self as serenity, FullEvent};

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
    gaijin: serenity::RoleId,
    gn_ch_id: serenity::ChannelId,
    member: serenity::RoleId,
    non_member: serenity::RoleId,
    old_member: serenity::RoleId,
    server: serenity::GuildId,
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

#[derive(Debug)]
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
    init_tracing_subscriber();

    // Run SQLx Migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Load token
    let token = secret!("DISCORD_TOKEN", secret_store);

    // Build Axum Router
    let router = axum::Router::new()
        .route("/up", axum::routing::get(routes::up))
        .route(
            "/verify",
            axum::routing::post({
                let pool = pool.clone();
                let key = secret_store
                    .get("VERIFY_KEY")
                    .context("VERIFY_KEY not found")?;
                move |body| routes::verify(pool, body, key)
            }),
        );

    // Build Poise Instance
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: all_commands(),
            event_handler: { |c, e, f, d| Box::pin(event_handler(c, e, f, d)) },
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

async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::GuildMemberAddition { new_member } => {
            tracing::info!("Member joined: {}", new_member.user.name);
        }
        FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(m),
        } => {
            tracing::info!("Interaction: {} by {}", m.data.custom_id, m.user.name);
            match m.data.custom_id.as_str() {
                "register.global" | "unregister.global" | "register.guild" | "unregister.guild" => {
                }
                "info" => verify::info(ctx, m).await?,
                "start" => verify::start(ctx, m, data, true).await?,
                "restart" => verify::start(ctx, m, data, false).await?,
                "login_1" => verify::login_1(ctx, m).await?,
                "login_2" => verify::login_2(ctx, m, data).await?,
                "login_3" => verify::login_3(ctx, m).await?,
                "login_4f" => verify::login_4(ctx, m, true).await?,
                "login_4n" => verify::login_4(ctx, m, false).await?,
                "login_5f" => verify::login_5(ctx, m, true).await?,
                "login_5n" => verify::login_5(ctx, m, false).await?,
                "membership_1" => verify::membership_1(ctx, m).await?,
                "membership_2f" => verify::membership_2(ctx, m, data, true).await?,
                "membership_2n" => verify::membership_2(ctx, m, data, false).await?,
                "manual_1" => verify::manual_1(ctx, m).await?,
                "manual_2f" => verify::manual_2(ctx, m, data, true).await?,
                "manual_2n" => verify::manual_2(ctx, m, data, false).await?,
                id if id.starts_with("verify-") => verify::manual_4(ctx, m, data, id).await?,
                _ => {
                    tracing::info!("Unknown interaction, printing:\n{m:#?}");
                    verify::unknown(ctx, m).await?;
                }
            }
        }
        FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Modal(m),
        } => {
            tracing::info!("Modal submit: {} by {}", m.data.custom_id, m.user.name);
            match m.data.custom_id.as_str() {
                "login_6f" => verify::login_6(ctx, m, data, true).await?,
                "login_6n" => verify::login_6(ctx, m, data, false).await?,
                "membership_3f" => verify::membership_3(ctx, m, data, true).await?,
                "membership_3n" => verify::membership_3(ctx, m, data, false).await?,
                "manual_3f" => verify::manual_3(ctx, m, data, true).await?,
                "manual_3n" => verify::manual_3(ctx, m, data, false).await?,
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn init_tracing_subscriber() {
    use tracing_subscriber as ts;
    use ts::prelude::*;
    ts::registry()
        .with(
            ts::fmt::layer()
                .without_time()
                .with_filter(ts::EnvFilter::new(
                    "info,nano=info,shuttle=info,serenity=off",
                )),
        )
        .with(
            ts::fmt::layer()
                .without_time()
                .fmt_fields(ts::fmt::format::debug_fn(|w, f, v| {
                    if f.name() == "message" {
                        write!(w, "{v:?}")
                    } else {
                        write!(w, "")
                    }
                }))
                .with_filter(ts::EnvFilter::new("off,serenity=info")),
        )
        .init();
}

fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        cmds::cmds(),
        cmds::setup(),
        cmds::count_members(),
        cmds::delete_member(),
        cmds::get_all_members(),
        cmds::get_member(),
        cmds::add_member(),
        cmds::insert_member_from_pending(),
        cmds::insert_member_from_manual(),
        cmds::nick(),
        cmds::edit_member(),
        cmds::refresh_non_members(),
        cmds::set_members_non_fresher(),
        cmds::count_pending(),
        cmds::delete_pending(),
        cmds::get_all_pending(),
        cmds::get_pending(),
        cmds::add_pending(),
        cmds::delete_all_pending(),
        cmds::count_manual(),
        cmds::delete_manual(),
        cmds::get_all_manual(),
        cmds::get_manual(),
        cmds::add_manual(),
        cmds::delete_all_manual(),
        cmds::whois(),
        cmds::count_gaijin(),
        cmds::delete_gaijin(),
        cmds::get_all_gaijin(),
        cmds::get_gaijin(),
        cmds::add_gaijin(),
        cmds::edit_gaijin(),
    ]
}
