use crate::{var, verify, Data, Error, Fresher};
use anyhow::Context as _;
use poise::serenity_prelude::{self as serenity, FullEvent};
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;

pub(crate) fn nanobot(pool: sqlx::SqlitePool) -> Result<poise::Framework<Data, Error>, Error> {
    // Build Bot Data
    let data = Data {
        au_ch_id: var!("AU_CHANNEL_ID", _),
        db: pool,
        ea_key: var!("EA_API_KEY"),
        ea_url: var!("EA_API_URL"),
        fresher_pg: var!("FRESHER_PG_ID", _),
        fresher_ug: var!("FRESHER_UG_ID", _),
        gaijin: var!("GAIJIN_ID", _),
        gn_ch_id: var!("GN_CHANNEL_ID", _),
        member: var!("MEMBER_ID", _),
        non_member: var!("NON_MEMBER_ID", _),
        old_member: var!("OLD_MEMBER_ID", _),
        server: var!("SERVER_ID", _),
    };

    // Build Poise Instance
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: crate::cmds::all_commands(),
            event_handler: { |c, e, f, d| Box::pin(event_handler(c, e, f, d)) },
            ..Default::default()
        })
        .setup(move |ctx, _, _| {
            Box::pin(async move {
                ctx.set_activity(Some(serenity::ActivityData::custom(
                    "Verifying members since 2023",
                )));
                Ok(data)
            })
        })
        .build();

    // Return NanoBot
    Ok(framework)
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
                "login_4n" => verify::login_4(ctx, m, Fresher::No).await?,
                "login_4p" => verify::login_4(ctx, m, Fresher::YesPg).await?,
                "login_4u" => verify::login_4(ctx, m, Fresher::YesUg).await?,
                "login_5n" => verify::login_5(ctx, m, Fresher::No).await?,
                "login_5p" => verify::login_5(ctx, m, Fresher::YesPg).await?,
                "login_5u" => verify::login_5(ctx, m, Fresher::YesUg).await?,
                "membership_1" => verify::membership_1(ctx, m).await?,
                "membership_2n" => verify::membership_2(ctx, m, data, Fresher::No).await?,
                "membership_2p" => verify::membership_2(ctx, m, data, Fresher::YesPg).await?,
                "membership_2u" => verify::membership_2(ctx, m, data, Fresher::YesUg).await?,
                "manual_1" => verify::manual_1(ctx, m).await?,
                "manual_2n" => verify::manual_2(ctx, m, data, Fresher::No).await?,
                "manual_2p" => verify::manual_2(ctx, m, data, Fresher::YesPg).await?,
                "manual_2u" => verify::manual_2(ctx, m, data, Fresher::YesUg).await?,
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
                "login_6n" => verify::login_6(ctx, m, data, Fresher::No).await?,
                "login_6p" => verify::login_6(ctx, m, data, Fresher::YesPg).await?,
                "login_6u" => verify::login_6(ctx, m, data, Fresher::YesUg).await?,
                "membership_3n" => verify::membership_3(ctx, m, data, Fresher::No).await?,
                "membership_3p" => verify::membership_3(ctx, m, data, Fresher::YesPg).await?,
                "membership_3u" => verify::membership_3(ctx, m, data, Fresher::YesUg).await?,
                "manual_3n" => verify::manual_3(ctx, m, data, Fresher::No).await?,
                "manual_3p" => verify::manual_3(ctx, m, data, Fresher::YesPg).await?,
                "manual_3u" => verify::manual_3(ctx, m, data, Fresher::YesUg).await?,
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) async fn init_db(db_url: &str) -> Result<sqlx::SqlitePool, Error> {
    use sqlx::migrate::MigrateDatabase;

    if !sqlx::Sqlite::database_exists(db_url).await? {
        sqlx::Sqlite::create_database(db_url).await?;
    }

    let pool = sqlx::SqlitePool::connect(db_url).await?;
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

pub(crate) fn init_tracing_subscriber() {
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

pub(crate) async fn shutdown_handler(token: CancellationToken) {
    let sig_int = ctrl_c();

    #[cfg(unix)]
    let mut signal = signal(SignalKind::terminate()).expect("SIGTERM");
    #[cfg(unix)]
    let sig_term = signal.recv();
    #[cfg(not(unix))]
    let sig_term = std::future::pending::<Option<()>>();

    tokio::select! {
        _ = sig_int => tracing::info!("Received SIGINT, initiating graceful shutdown"),
        _ = sig_term => tracing::info!("Received SIGTERM, initiating graceful shutdown"),
    }

    token.cancel();
}
