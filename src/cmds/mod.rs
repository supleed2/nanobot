use crate::{ACtx, Data, Error};
use poise::serenity_prelude::{self as serenity, CreateActionRow, CreateButton, CreateMessage};
use poise::Modal;

pub(crate) mod members;
pub(crate) use members::*;

pub(crate) mod pending;
pub(crate) use pending::*;

pub(crate) mod manual;
pub(crate) use manual::*;

pub(crate) mod edit_members;
pub(crate) use edit_members::*;

pub(crate) mod whois;
pub(crate) use whois::*;

pub(crate) mod extras;
pub(crate) use extras::*;

/// Buttons to (de-)register application commands globally or by guild
#[tracing::instrument(skip_all)]
#[poise::command(prefix_command, owners_only)]
pub(crate) async fn cmds(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    tracing::info!("{}", ctx.author().name);
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Send (customisable) verification introduction message in specified channel
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn setup(
    ctx: ACtx<'_>,
    #[description = "Channel to send verification introduction message in"]
    #[channel_types("Text", "News")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    #[derive(Modal)]
    struct Setup {
        #[name = "Contents of the verification intro message"]
        #[placeholder = "Welcome... Click the \"Begin\" button below to start verification"]
        #[paragraph]
        #[max_length = 500]
        message: String,
        #[name = "Emoji for the start button"]
        #[placeholder = "🚀"]
        #[max_length = 4]
        emoji: Option<String>,
        #[name = "Label for the start button"]
        #[placeholder = "Begin"]
        #[max_length = 80]
        text: Option<String>,
    }

    tracing::info!("{} {}", ctx.author().name, channel.name());

    if let Some(Setup {
        message,
        emoji,
        text,
    }) = Setup::execute(ctx).await?
    {
        ctx.say(format!("Sending intro message in {channel}"))
            .await?;
        let emoji = emoji.unwrap_or_default().chars().next().unwrap_or('🚀');
        channel
            .send_message(
                ctx.http(),
                CreateMessage::new()
                    .content(message)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new("info")
                            .style(serenity::ButtonStyle::Secondary)
                            .emoji('📖')
                            .label("More info"),
                        CreateButton::new("start")
                            .style(serenity::ButtonStyle::Primary)
                            .emoji(emoji)
                            .label(text.unwrap_or("Begin".to_string())),
                    ])]),
            )
            .await?;
    } else {
        ctx.say("Modal timed out, try again...").await?;
    }
    Ok(())
}
