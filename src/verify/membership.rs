use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use poise::Modal;

const MEMBERSHIP_INTRO: &str = indoc::indoc! {"
    To use automatic verification via Membership:
    - Enter your Union order number (from this academic year)
    - Enter your Imperial shortcode
    - Enter your preferred name for Nano whois commands
    - Your shortcode will then be connected to your Discord Account by Nano

    First, are you a fresher?
"};

pub(crate) async fn membership_1(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content(MEMBERSHIP_INTRO).components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(serenity::ButtonStyle::Danger)
                                .emoji('🔙')
                                .custom_id("restart")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Success)
                                .emoji('✅')
                                .label("Fresher")
                                .custom_id("membership_2f")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                                .emoji('❌')
                                .label("Non-fresher")
                                .custom_id("membership_2n")
                        })
                    })
                })
            })
    })
    .await?;
    Ok(())
}

#[derive(Modal)]
#[name = "ICAS Membership Verification"]
struct Membership {
    #[name = "ICAS Membership Union Order Number"]
    #[placeholder = "1234567"]
    order: String,
    #[name = "Imperial Shortcode"]
    #[placeholder = "ab1234"]
    shortcode: String,
    #[name = "Preferred name for Nano whois commands"]
    #[placeholder = "Firstname Lastname"]
    nickname: String,
}

pub(crate) async fn membership_2(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    // Delete from pending if exists
    let _ = crate::db::delete_pending_by_id(&data.db, m.user.id.0 as i64).await;

    // Delete from manual if exists
    let _ = crate::db::delete_manual_by_id(&data.db, m.user.id.0 as i64).await;

    m.create_interaction_response(&ctx.http, |i| {
        *i = Membership::create(
            None,
            if fresher {
                "membership_3f".to_string()
            } else {
                "membership_3n".to_string()
            },
        );
        i
    })
    .await?;
    Ok(())
}

pub(crate) async fn membership_3(
    ctx: &serenity::Context,
    m: &serenity::ModalSubmitInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    match Membership::parse(m.data.clone()) {
        Ok(Membership {
            order,
            shortcode,
            nickname,
        }) => {
            let members = match crate::ea::get_members_list(&data.ea_key, &data.ea_url).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error: {e}");
                    m.create_interaction_response(&ctx.http, |i| {
                        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.content("Sorry, getting membership data failed. Please try again")
                                    .ephemeral(true)
                            })
                    })
                    .await?;
                    return Ok(());
                }
            };
            let member = match members
                .iter()
                .find(|&member| member.order_no.to_string() == order && member.login == shortcode)
            {
                Some(m) => m,
                None => {
                    m.create_interaction_response(&ctx.http, |i| {
                        let msg = "Sorry, your order was not found, please check the \
                            order number and that it is for your current year's membership";
                        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| d.content(msg).ephemeral(true))
                    })
                    .await?;
                    return Ok(());
                }
            };
            if crate::db::insert_member(
                &data.db,
                crate::Member {
                    discord_id: m.user.id.0 as i64,
                    shortcode,
                    nickname,
                    realname: format!("{} {}", member.first_name, member.surname),
                    fresher,
                },
            )
            .await
            .is_ok()
            {
                println!(
                    "{} ({}) added via membership{}",
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
                            e.thumbnail(m.user.avatar_url().unwrap_or(
                                "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                            ))
                            .title("Member verified via membership")
                            .description(&m.user)
                            .field("Fresher", fresher, true)
                            .timestamp(serenity::Timestamp::now())
                        })
                    })
                    .await?;
                data.gn_ch_id
                    .send_message(&ctx.http, |cm| {
                        cm.content(format!(
                            "Welcome to ICAS {}, if you have any questions, feel free \
                            to ping a committee member{}!",
                            m.user,
                            if fresher {
                                ", and look out for other freshers in green"
                            } else {
                                ""
                            }
                        ))
                    })
                    .await?;
                return Ok(());
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    };
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|d| {
                d.content("Sorry, something went wrong. Please try again")
                    .ephemeral(true)
            })
    })
    .await?;
    Ok(())
}
