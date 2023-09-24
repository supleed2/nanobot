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
    let member = secret_store
        .get("MEMBER_ID")
        .expect("MEMBER_ID not found")
        .parse()
        .expect("MEMBER_ID not valid u64");

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
                    member,
                })
            })
        });
