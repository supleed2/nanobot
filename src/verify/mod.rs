use crate::{Data, Error};
use poise::serenity_prelude as serenity;

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

pub(crate) async fn info(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|d| d.content(INFO_MSG).ephemeral(true))
    })
    .await?;
    Ok(())
}

pub(crate) async fn unknown(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
			.interaction_response_data(|d| {
				d.content("Sorry, something went wrong. Please try again or message <@99217900254035968> for help")
					.ephemeral(true)
		})
    })
    .await?;
    Ok(())
}

const START_MSG: &str = indoc::indoc! {"
    There are 3 available methods for verification.
    - 🚀 Automatic verification via Imperial Login (Quickest)
    - ✈️ Automatic verification via ICAS Membership (Easiest)
    - 🚗 Manual verification, eg. using College ID Card or Acceptance Letter
"};

pub(crate) async fn start(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
    data: &Data,
    init: bool,
) -> Result<(), Error> {
    // Check if user is already verified
    if let Some(member) = crate::db::get_member_by_id(&data.db, m.user.id.0 as i64).await? {
        let mut mm = m.member.clone().unwrap();
        apply_role(ctx, &mut mm, data.member).await?;
        if member.fresher {
            apply_role(ctx, &mut mm, data.fresher).await?;
        }
        m.create_interaction_response(&ctx.http, |i| {
            i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.content("Welcome, you're already verified, re-applied your roles!")
                        .ephemeral(true)
                })
        })
        .await?
    } else {
        m.create_interaction_response(&ctx.http, |i| {
            i.kind(if init {
                serenity::InteractionResponseType::ChannelMessageWithSource
            } else {
                serenity::InteractionResponseType::UpdateMessage
            })
            .interaction_response_data(|d| {
                d.content(START_MSG).ephemeral(true).components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                                .emoji('🚀')
                                .label("Login")
                                .custom_id("login_1")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Secondary)
                                .emoji(serenity::ReactionType::Unicode("✈️".to_string()))
                                .label("Membership")
                                .custom_id("membership_1")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Secondary)
                                .emoji('🚗')
                                .label("Manual")
                                .custom_id("manual_1")
                        })
                    })
                })
            })
        })
        .await?
    };
    Ok(())
}

pub(crate) async fn apply_role(
    ctx: &serenity::Context,
    member: &mut serenity::Member,
    role: serenity::RoleId,
) -> Result<(), Error> {
    Ok(member.add_role(&ctx.http, role).await?)
}

pub(crate) async fn remove_role(
    ctx: &serenity::Context,
    member: &mut serenity::Member,
    role: serenity::RoleId,
) -> Result<(), Error> {
    Ok(member.remove_role(&ctx.http, role).await?)
}
