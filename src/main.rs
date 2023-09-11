use anyhow::Context as _;
use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;

/// Buttons to (de-)register application commands globally or by guild
#[poise::command(prefix_command)]
async fn register(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Send (customisable) verification introduction message in specified channel
#[poise::command(slash_command)]
async fn setup(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Channel to send verification introduction message in"]
    #[channel_types("Text", "News")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let resp = format!("You selected channel: {}", channel);
    ctx.say(resp).await?;
    Ok(())
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
