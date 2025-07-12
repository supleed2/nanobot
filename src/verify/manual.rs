use crate::{db, verify, Data, Error, ManualMember};
use poise::serenity_prelude::{
    self as serenity, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage,
};
use poise::Modal;

const MANUAL_INTRO: &str = indoc::indoc! {"
    Submit details to be manually checked by a committee member:
    - Your Imperial Shortcode
    - Your First and Last Names as on your Imperial record
    - Preferred First and Last Names for the Nano whois command
    - URL to proof of being an Imperial student, e.g. photo of College ID Card \
        or screenshot of College Acceptance Letter, if you need to upload this, \
        you can send it in a DM and then copy the image URL

    We try to respond quickly but this may take a day or two during busy term times :)

    First, are you a fresher?
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn manual_1(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .content(MANUAL_INTRO)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("restart")
                        .style(serenity::ButtonStyle::Danger)
                        .emoji('🔙'),
                    CreateButton::new("manual_2f")
                        .style(serenity::ButtonStyle::Success)
                        .emoji('✅')
                        .label("Fresher"),
                    CreateButton::new("manual_2n")
                        .style(serenity::ButtonStyle::Primary)
                        .emoji('❌')
                        .label("Non-fresher"),
                ])]),
        ),
    )
    .await?;
    Ok(())
}

#[derive(Modal)]
#[name = "Manual Verification"]
struct Manual {
    #[name = "Imperial Shortcode"]
    #[placeholder = "ab1234"]
    shortcode: String,
    #[name = "Name as on Imperial record"]
    #[placeholder = "Firstname Lastname"]
    realname: String,
    #[name = "URL to proof image"]
    #[placeholder = "E.g. photo of College ID Card \
    or screenshot of College Acceptance Letter"]
    url: String,
    #[name = "Preferred name for Nano whois commands"]
    #[placeholder = "Firstname Lastname"]
    nickname: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn manual_2(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    // Delete from manual if exists
    let _ = db::delete_manual_by_id(&data.db, m.user.id.into()).await;

    m.create_response(
        &ctx.http,
        Manual::create(
            None,
            if fresher {
                "manual_3f".to_string()
            } else {
                "manual_3n".to_string()
            },
        ),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn manual_3(
    ctx: &serenity::Context,
    m: &serenity::ModalInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    match Manual::parse(m.data.clone()) {
        Ok(Manual {
            shortcode,
            realname,
            url,
            nickname,
        }) => {
            if ::url::Url::parse(&url).is_err() {
                m.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("The url provided is invalid, please try again")
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            }

            // Delete from pending if exists
            let _ = db::delete_pending_by_id(&data.db, m.user.id.into()).await?;

            let prompt_sent = data
                .au_ch_id
                .send_message(
                    &ctx.http,
                    CreateMessage::new()
                        .embed(
                            CreateEmbed::new()
                                .title("New verification request from")
                                .thumbnail(m.user.face())
                                .description(m.user.to_string())
                                .field("Real Name (To be checked)", &realname, true)
                                .field("Imperial Shortcode (To be checked", &shortcode, true)
                                .field("Fresher (To be checked)", fresher.to_string(), true)
                                .field("Nickname (Nano whois commands)", &nickname, true)
                                .field("Verification URL (Also displayed below)", &url, true)
                                .image(&url)
                                .timestamp(serenity::Timestamp::now()),
                        )
                        .components(vec![CreateActionRow::Buttons(vec![
                            CreateButton::new(format!("verify-y-{}", m.user.id))
                                .style(serenity::ButtonStyle::Success)
                                .emoji('✅')
                                .label("Accept"),
                            CreateButton::new(format!("verify-n-{}", m.user.id))
                                .style(serenity::ButtonStyle::Danger)
                                .emoji('❎')
                                .label("Deny"),
                        ])]),
                )
                .await
                .is_ok();

            let inserted = db::insert_manual(
                &data.db,
                ManualMember {
                    discord_id: m.user.id.into(),
                    shortcode,
                    nickname,
                    realname,
                    fresher,
                },
            )
            .await
            .is_ok();

            let msg = if prompt_sent {
                if inserted {
                    "Thanks, your verification request has been sent, we'll try to get back to you quickly!"
                } else {
                    "Thanks, your verification request has been sent, but there was an issue, please ask a Committee member to take a look!"
                }
            } else {
                "Sending your verification request failed, please try again."
            };

            m.create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(msg)
                        .components(vec![]),
                ),
            )
            .await?;
            return Ok(());
        }
        Err(e) => tracing::error!("{e}"),
    }
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Sorry, something went wrong. Please try again")
                .ephemeral(true),
        ),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn manual_4(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    data: &Data,
    id: &str,
) -> Result<(), Error> {
    let verify = matches!(id.chars().nth(7), Some('y'));
    let user = id
        .chars()
        .skip(9)
        .collect::<String>()
        .parse::<u64>()
        .map(serenity::UserId::new)
        .unwrap_or_default()
        .to_user(ctx)
        .await
        .unwrap_or_default();

    let mut member = data.server.member(&ctx.http, &user).await?;

    if verify {
        match db::insert_member_from_manual(&data.db, user.id.into()).await {
            Ok(mm) => {
                tracing::info!(
                    "{} ({}) added via manual{}",
                    user.name,
                    user.id,
                    if mm.fresher { " (fresher)" } else { "" }
                );
                verify::apply_role(ctx, &mut member, data.member).await?;
                if mm.fresher {
                    verify::apply_role(ctx, &mut member, data.fresher).await?;
                }
                m.create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .components(vec![])
                            .embed(
                                CreateEmbed::new()
                                    .thumbnail(user.face())
                                    .title("Member verified via manual")
                                    .description(user.to_string())
                                    .field("Fresher", mm.fresher.to_string(), true)
                                    .field("Nickname", mm.nickname, true)
                                    .field("Name", mm.realname, true)
                                    .timestamp(serenity::Timestamp::now()),
                            ),
                    ),
                )
                .await?;
                let _ = member.remove_role(&ctx.http, data.non_member).await;
                if member.roles.contains(&data.old_member) {
                    verify::remove_role(ctx, &mut member, data.old_member).await?;
                } else {
                    verify::welcome_user(&ctx.http, &data.gn_ch_id, &user, mm.fresher).await?;
                }
            }
            Err(e) => {
                tracing::error!("{e}");
                m.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("Failed to add user {user} to member database")),
                    ),
                )
                .await?;
            }
        }
    } else {
        db::delete_manual_by_id(&data.db, user.id.into()).await?;
        tracing::info!("{} ({}) denied via manual", user.name, user.id);
        m.create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .components(vec![])
                    .embed(
                        CreateEmbed::new()
                            .title("Member denied via manual")
                            .description(user.to_string())
                            .thumbnail(user.face())
                            .timestamp(serenity::Timestamp::now()),
                    ),
            ),
        )
        .await?;
    }

    Ok(())
}
