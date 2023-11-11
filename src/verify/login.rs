use crate::{Data, Error};
use poise::serenity_prelude as serenity;
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
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content(LOGIN_INTRO).components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(serenity::ButtonStyle::Danger)
                                .emoji('🔙')
                                .custom_id("restart")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Link)
                                .emoji('🚀')
                                .label("Login Here")
                                .url(format!("https://icas.8bitsqu.id/verify?id={}", m.user.id.0))
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Secondary)
                                .emoji('👉')
                                .label("Then continue")
                                .custom_id("login_2")
                        })
                    })
                })
            })
    })
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
    m: &serenity::MessageComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    match crate::db::get_pending_by_id(&data.db, m.user.id.into()).await {
        Err(e) => {
            tracing::error!("{e}");
            m.create_interaction_response(&ctx.http, |i| {
                i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.content("Sorry, something went wrong. Please try again")
                            .ephemeral(true)
                    })
            })
            .await?;
        }
        Ok(None) => {
            m.create_interaction_response(&ctx.http, |i| {
                i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.content("Error, have you completed login verification via the link?")
                            .ephemeral(true)
                    })
            })
            .await?;
        }
        Ok(Some(_)) => {
            m.create_interaction_response(&ctx.http, |i| {
                i.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content(LOGIN_FORM).components(|c| {
                            c.create_action_row(|a| {
                                a.create_button(|b| {
                                    b.style(serenity::ButtonStyle::Danger)
                                        .emoji('🔙')
                                        .custom_id("login_1")
                                })
                                .create_button(|b| {
                                    b.style(serenity::ButtonStyle::Primary)
                                        .emoji('📑')
                                        .label("Form")
                                        .custom_id("login_3")
                                })
                            })
                        })
                    })
            })
            .await?;
        }
    };
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_3(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content("Are you a fresher?").components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(serenity::ButtonStyle::Danger)
                                .emoji('🔙')
                                .custom_id("login_2")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Success)
                                .emoji('✅')
                                .label("Fresher")
                                .custom_id("login_4f")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                                .emoji('❌')
                                .label("Non-fresher")
                                .custom_id("login_4n")
                        })
                    })
                })
            })
    })
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_4(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
    fresher: bool,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content("And a preferred name for Nano whois commands")
                    .components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.style(serenity::ButtonStyle::Danger)
                                    .emoji('🔙')
                                    .custom_id("login_3")
                            })
                            .create_button(|b| {
                                b.style(serenity::ButtonStyle::Primary)
                                    .emoji('💬')
                                    .label("Name")
                                    .custom_id(if fresher { "login_5f" } else { "login_5n" })
                            })
                        })
                    })
            })
    })
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
    m: &serenity::MessageComponentInteraction,
    fresher: bool,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        *i = Nickname::create(
            None,
            if fresher {
                "login_6f".to_string()
            } else {
                "login_6n".to_string()
            },
        );
        i
    })
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn login_6(
    ctx: &serenity::Context,
    m: &serenity::ModalSubmitInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    match Nickname::parse(m.data.clone()) {
        Ok(Nickname { nickname }) => {
            // Delete from manual if exists
            let _ = crate::db::delete_manual_by_id(&data.db, m.user.id.into()).await;

            match crate::db::insert_member_from_pending(
                &data.db,
                m.user.id.into(),
                &nickname,
                fresher,
            )
            .await
            {
                Ok(p) => {
                    tracing::info!(
                        "{} ({}) added via login{}",
                        m.user.name,
                        m.user.id,
                        if fresher { " (fresher)" } else { "" }
                    );
                    let mut mm = m.member.clone().unwrap();
                    crate::verify::apply_role(ctx, &mut mm, data.member).await?;
                    if fresher {
                        crate::verify::apply_role(ctx, &mut mm, data.fresher).await?;
                    }
                    m.create_interaction_response(&ctx.http, |i| {
                        i.kind(serenity::InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|d| {
                                d.content(if fresher {
                                    "Congratulations, you have completed verification and now \
                                    have access to the ICAS Discord and freshers thread"
                                } else {
                                    "Congratulations, you have completed verification and now \
                                    have access to the ICAS Discord"
                                })
                                .components(|c| c)
                            })
                    })
                    .await?;
                    data.au_ch_id
                        .send_message(&ctx.http, |cm| {
                            cm.add_embed(|e| {
                                e.thumbnail(
                                    m.user.avatar_url().unwrap_or(super::AVATAR.to_string()),
                                )
                                .title("Member verified via login")
                                .description(&m.user)
                                .field("Fresher", fresher, true)
                                .field("Nickname", nickname, true)
                                .field("Name", p.realname, true)
                                .timestamp(serenity::Timestamp::now())
                            })
                        })
                        .await?;
                    let _ = mm.remove_role(&ctx.http, data.non_member).await;
                    if mm.roles.contains(&data.old_member) {
                        crate::verify::remove_role(ctx, &mut mm, data.old_member).await?;
                    } else {
                        crate::verify::welcome_user(&ctx.http, &data.gn_ch_id, &m.user, fresher)
                            .await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Error: {e}");
                    m.create_interaction_response(&ctx.http, |i| {
                        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.content("Sorry, something went wrong. Please try again")
                                    .ephemeral(true)
                            })
                    })
                    .await?;
                }
            }
        }
        Err(e) => {
            tracing::error!("{e}");
        }
    };
    Ok(())
}
