use crate::{db, Data, Error};
use poise::serenity_prelude::{
    self as serenity, CacheHttp, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage,
};

pub(crate) mod login;
pub(crate) use login::*;

pub(crate) mod membership;
pub(crate) use membership::*;

pub(crate) mod manual;
pub(crate) use manual::*;

const INFO_MSG: &str = indoc::indoc! {"
    Nano is a Discord bot written with serenity-rs/poise and tokio-rs/axum.

    It allows members and Imperial students to automatically verify themselves and gain access to the ICAS Discord server.

    If you have any questions, feel free to ping or message <@99217900254035968>
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn info(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(INFO_MSG)
                .ephemeral(true),
        ),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn unknown(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(
                    "Sorry, something went wrong. Please try again \
                    or message <@99217900254035968> for help",
                )
                .ephemeral(true),
        ),
    )
    .await?;
    Ok(())
}

const START_MSG: &str = indoc::indoc! {"
    There are 3 available methods for verification.
    - 🚀 Automatic verification via Imperial Login (Quickest)
    - ✈️ Automatic verification via ICAS Membership (Easiest)
    - 🚗 Manual verification, eg. using College ID Card or Acceptance Letter
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn start(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    data: &Data,
    init: bool,
) -> Result<(), Error> {
    // Check if user is already verified
    if let Some(member) = db::get_member_by_id(&data.db, m.user.id.into()).await? {
        let mut mm = m.member.clone().unwrap();
        apply_role(ctx, &mut mm, data.member).await?;
        if member.fresher {
            apply_role(ctx, &mut mm, data.fresher).await?;
        }
        m.create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Welcome, you're already verified, re-applied your roles!")
                    .ephemeral(true),
            ),
        )
        .await?;
    } else {
        let irm = CreateInteractionResponseMessage::new()
            .content(START_MSG)
            .ephemeral(true)
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("login_1")
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('🚀')
                    .label("Login"),
                CreateButton::new("membership_1")
                    .style(serenity::ButtonStyle::Secondary)
                    .emoji(serenity::ReactionType::Unicode("✈️".to_string()))
                    .label("Membership"),
                CreateButton::new("manual_1")
                    .style(serenity::ButtonStyle::Secondary)
                    .emoji('🚗')
                    .label("Manual"),
            ])]);
        m.create_response(
            &ctx.http,
            if init {
                CreateInteractionResponse::Message(irm)
            } else {
                CreateInteractionResponse::UpdateMessage(irm)
            },
        )
        .await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn apply_role(
    ctx: &serenity::Context,
    member: &mut serenity::Member,
    role: serenity::RoleId,
) -> Result<(), Error> {
    Ok(member.add_role(&ctx.http, role).await?)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn remove_role(
    ctx: &serenity::Context,
    member: &mut serenity::Member,
    role: serenity::RoleId,
) -> Result<(), Error> {
    Ok(member.remove_role(&ctx.http, role).await?)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn welcome_user(
    http: impl CacheHttp,
    channel: &serenity::ChannelId,
    user: &serenity::User,
    fresher: bool,
) -> Result<(), Error> {
    channel
        .send_message(
            http,
            CreateMessage::new().content(format!(
                "Welcome to ICAS {user}, if you have any questions, \
                    feel free to ping a committee member{}!",
                if fresher {
                    ", and look out for other freshers in green"
                } else {
                    ""
                }
            )),
        )
        .await?;
    Ok(())
}
