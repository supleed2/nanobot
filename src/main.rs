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
    gn_ch_id: serenity::ChannelId,
    member: serenity::RoleId,
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

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> Result<service::NanoBot, shuttle_runtime::Error> {
    // Set Up Tracing Subscriber
    use tracing_subscriber as ts;
    use ts::prelude::*;
    ts::registry()
        .with(
            ts::fmt::layer()
                .without_time()
                .with_filter(ts::EnvFilter::new(
                    "info,nano=info,shuttle=trace,serenity=off",
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
    tracing::info!("Tracing Subscriber Set Up");

    // Run SQLx Migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Load secrets
    let au_ch_id = secret_store
        .get("AU_CHANNEL_ID")
        .expect("AU_CHANNEL_ID not found")
        .parse()
        .expect("AU_CHANNEL_ID not valid u64");
    let ea_key = secret_store
        .get("EA_API_KEY")
        .expect("EA_API_KEY not found");
    let ea_url = secret_store
        .get("EA_API_URL")
        .expect("EA_API_URL not found");
    let token = secret_store
        .get("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN not found");
    let fresher = secret_store
        .get("FRESHER_ID")
        .expect("FRESHER_ID not found")
        .parse()
        .expect("FRESHER_ID not valid u64");
    let gn_ch_id = secret_store
        .get("GN_CHANNEL_ID")
        .expect("GN_CHANNEL_ID not found")
        .parse()
        .expect("GN_CHANNEL_ID not valid u64");
    let member = secret_store
        .get("MEMBER_ID")
        .expect("MEMBER_ID not found")
        .parse()
        .expect("MEMBER_ID not valid u64");
    let old_member = secret_store
        .get("OLD_MEMBER_ID")
        .expect("OLD_MEMBER_ID not found")
        .parse()
        .expect("OLD_MEMBER_ID not valid u64");
    let server = secret_store
        .get("SERVER_ID")
        .expect("SERVER_ID not found")
        .parse::<u64>()
        .expect("SERVER_ID not valid u64")
        .into();
    tracing::info!("Secrets loaded");

    // Build Axum Router
    let router = axum::Router::new().route(
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
    let discord = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                cmds::cmds(),
                cmds::setup(),
                cmds::count_members(),
                cmds::delete_member(),
                cmds::get_all_members(),
                cmds::get_member(),
                cmds::add_member(),
                cmds::insert_member_from_pending(),
                cmds::insert_member_from_manual(),
                cmds::edit_member(),
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
            ],
            event_handler: { |c, e, f, d| Box::pin(event_handler(c, e, f, d)) },
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(move |ctx, _, _| {
            Box::pin(async move {
                ctx.set_activity(serenity::Activity::competing("autoverification"))
                    .await;
                Ok(Data {
                    au_ch_id,
                    db: pool,
                    ea_key,
                    ea_url,
                    fresher,
                    gn_ch_id,
                    member,
                    old_member,
                    server,
                })
            })
        });

    // Return NanoBot
    Ok(service::NanoBot { discord, router })
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::GuildMemberAddition { new_member } => {
            tracing::info!("Member joined: {}", new_member.user.name);
        }
        poise::Event::InteractionCreate {
            interaction: serenity::Interaction::MessageComponent(m),
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
        poise::Event::InteractionCreate {
            interaction: serenity::Interaction::ModalSubmit(m),
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
