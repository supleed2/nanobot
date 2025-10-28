use crate::{db, verify, Data, Error, Fresher};
use poise::serenity_prelude::{
    self as serenity, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage,
};
use poise::Modal;

const LOGIN_INTRO: &str = indoc::indoc! {"
    To use automatic verification via Imperial Login:
    - Open the link provided and login using your shortcode
    - Your account will be checked and then the login details immediately discarded
    - Your shortcode will then be connected to your Discord Account by Nano

    You can then complete the remaining details in the next step!
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn login_1(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let verify_url = format!("https://icas.8bitsqu.id/verify?id={}", m.user.id.get());
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .content(LOGIN_INTRO)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("restart")
                        .style(serenity::ButtonStyle::Danger)
                        .emoji('🔙'),
                    CreateButton::new_link(verify_url)
                        .emoji('🚀')
                        .label("Login Here"),
                    CreateButton::new("login_2")
                        .style(serenity::ButtonStyle::Secondary)
                        .emoji('👉')
                        .label("Then continue"),
                ])]),
        ),
    )
    .await?;
    Ok(())
}

const LOGIN_FORM: &str = indoc::indoc! {"
    Congratulations, your Imperial shortcode has been connected to your Discord Account by Nano!

    The last step is a short form with some extra details
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn login_2(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    match db::get_pending_by_id(&data.db, m.user.id.into()).await {
        Err(e) => {
            tracing::error!("{e}");
            m.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Sorry, something went wrong. Please try again")
                        .ephemeral(true),
                ),
            )
            .await?;
        }
        Ok(None) => {
            m.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Error, have you completed login verification via the link?")
                        .ephemeral(true),
                ),
            )
            .await?;
        }
        Ok(Some(_)) => {
            m.create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(LOGIN_FORM)
                        .components(vec![CreateActionRow::Buttons(vec![
                            CreateButton::new("login_1")
                                .style(serenity::ButtonStyle::Danger)
                                .emoji('🔙'),
                            CreateButton::new("login_3")
                                .style(serenity::ButtonStyle::Primary)
                                .emoji('📑')
                                .label("Form"),
                        ])]),
                ),
            )
            .await?;
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_3(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .content("Are you a fresher?")
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("login_2")
                        .style(serenity::ButtonStyle::Danger)
                        .emoji('🔙'),
                    CreateButton::new("login_4u")
                        .style(serenity::ButtonStyle::Success)
                        .emoji('✅')
                        .label("Fresher"),
                    CreateButton::new("login_4n")
                        .style(serenity::ButtonStyle::Primary)
                        .emoji('❌')
                        .label("Non-fresher"),
                    CreateButton::new("login_4p")
                        .style(serenity::ButtonStyle::Success)
                        .emoji('🎓')
                        .label("Postgraduate fresher"),
                ])]),
        ),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_4(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    fresher: Fresher,
) -> Result<(), Error> {
    let next = match fresher {
        Fresher::No => "login_5n",
        Fresher::YesPg => "login_5p",
        Fresher::YesUg => "login_5u",
    };

    m.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .content("And a preferred name for Nano whois commands")
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("login_3")
                        .style(serenity::ButtonStyle::Danger)
                        .emoji('🔙'),
                    CreateButton::new(next)
                        .style(serenity::ButtonStyle::Primary)
                        .emoji('💬')
                        .label("Name"),
                ])]),
        ),
    )
    .await?;
    Ok(())
}

#[derive(Modal)]
#[name = "Preferred Name"]
struct Nickname {
    #[name = "Preferred name for Nano whois commands"]
    #[placeholder = "Firstname Lastname"]
    nickname: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_5(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    fresher: Fresher,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        Nickname::create(
            None,
            match fresher {
                Fresher::No => "login_6n".to_string(),
                Fresher::YesPg => "login_6p".to_string(),
                Fresher::YesUg => "login_6u".to_string(),
            },
        ),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_6(
    ctx: &serenity::Context,
    m: &serenity::ModalInteraction,
    data: &Data,
    fresher: Fresher,
) -> Result<(), Error> {
    match Nickname::parse(m.data.clone()) {
        Ok(Nickname { nickname }) => {
            // Delete from manual if exists
            let _ = db::delete_manual_by_id(&data.db, m.user.id.into()).await;

            match db::insert_member_from_pending(&data.db, m.user.id.into(), &nickname, fresher)
                .await
            {
                Ok(p) => {
                    tracing::info!(
                        "{} ({}) added via login ({})",
                        m.user.name,
                        m.user.id,
                        fresher
                    );
                    let mut mm = m.member.clone().unwrap();
                    verify::apply_role(ctx, &mut mm, data.member).await?;
                    match fresher {
                        Fresher::No => {}
                        Fresher::YesPg => verify::apply_role(ctx, &mut mm, data.fresher_pg).await?,
                        Fresher::YesUg => verify::apply_role(ctx, &mut mm, data.fresher_ug).await?,
                    }
                    let msg = if matches!(fresher, Fresher::No) {
                        "Congratulations, you have completed verification and now \
                        have access to the ICAS Discord"
                    } else {
                        "Congratulations, you have completed verification and now \
                        have access to the ICAS Discord and freshers thread"
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
                    data.au_ch_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::new().embed(
                                CreateEmbed::new()
                                    .thumbnail(m.user.face())
                                    .title("Member verified via login")
                                    .description(m.user.to_string())
                                    .field("Fresher", fresher.to_string(), true)
                                    .field("Nickname", nickname, true)
                                    .field("Name", p.realname, true)
                                    .timestamp(serenity::Timestamp::now()),
                            ),
                        )
                        .await?;
                    let _ = mm.remove_role(&ctx.http, data.non_member).await;
                    if mm.roles.contains(&data.old_member) {
                        verify::remove_role(ctx, &mut mm, data.old_member).await?;
                    } else {
                        verify::welcome_user(&ctx.http, &data.gn_ch_id, &m.user, fresher).await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Error: {e}");
                    m.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Sorry, something went wrong. Please try again")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                }
            }
        }
        Err(e) => {
            tracing::error!("{e}");
        }
    }
    Ok(())
}
