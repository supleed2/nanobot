use crate::{Data, Error};
use poise::serenity_prelude as serenity;
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

pub(crate) async fn manual_1(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
) -> Result<(), Error> {
    m.create_interaction_response(&ctx.http, |i| {
        i.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content(MANUAL_INTRO).components(|c| {
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
                                .custom_id("manual_2f")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                                .emoji('❌')
                                .label("Non-fresher")
                                .custom_id("manual_2n")
                        })
                    })
                })
            })
    })
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

pub(crate) async fn manual_2(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    // Delete from manual if exists
    let _ = crate::db::delete_manual_by_id(&data.db, m.user.id.0 as i64).await;

    m.create_interaction_response(&ctx.http, |i| {
        *i = Manual::create(
            None,
            if fresher {
                "manual_3f".to_string()
            } else {
                "manual_3n".to_string()
            },
        );
        i
    })
    .await?;
    Ok(())
}

pub(crate) async fn manual_3(
    ctx: &serenity::Context,
    m: &serenity::ModalSubmitInteraction,
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
                m.create_interaction_response(&ctx.http, |i| {
                    i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.content("The url provided is invalid, please try again")
                                .ephemeral(true)
                        })
                })
                .await?;
                return Ok(());
            }

            // Delete from pending if exists
            let _ = crate::db::delete_pending_by_id(&data.db, m.user.id.0 as i64).await?;

            let prompt_sent = data
                .au_ch_id
                .send_message(&ctx.http, |cm| {
                    cm.add_embed(|e| {
                        e.title("New verification request from")
                            .thumbnail(m.user.avatar_url().unwrap_or(
                                "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                            ))
                            .description(&m.user)
                            .field("Real Name (To be checked)", &realname, true)
                            .field("Imperial Shortcode (To be checked", &shortcode, true)
                            .field("Fresher (To be checked)", fresher, true)
                            .field("Nickname (Nano whois commands)", &nickname, true)
                            .field("Verification URL (Also displayed below)", &url, true)
                            .image(&url)
                            .timestamp(serenity::Timestamp::now())
                    })
                    .components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.style(serenity::ButtonStyle::Success)
                                    .emoji('✅')
                                    .label("Accept")
                                    .custom_id(format!("verify-y-{}", m.user.id))
                            })
                            .create_button(|b| {
                                b.style(serenity::ButtonStyle::Danger)
                                    .emoji('❎')
                                    .label("Deny")
                                    .custom_id(format!("verify-n-{}", m.user.id))
                            })
                        })
                    })
                })
                .await
                .is_ok();

            let inserted = crate::db::insert_manual(
                &data.db,
                crate::ManualMember {
                    discord_id: m.user.id.0 as i64,
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

            m.create_interaction_response(&ctx.http, |i| {
                i.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| d.content(msg).components(|c| c))
            })
            .await?;
            return Ok(());
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

pub(crate) async fn manual_4(
    ctx: &serenity::Context,
    m: &serenity::MessageComponentInteraction,
    data: &Data,
    id: &str,
) -> Result<(), Error> {
    let verify = match id.chars().nth(7) {
        Some('y') => true,
        Some('n') => false,
        _ => false,
    };

    let user = id
        .chars()
        .skip(9)
        .collect::<String>()
        .parse::<u64>()
        .map(serenity::UserId)
        .unwrap_or_default()
        .to_user(ctx)
        .await
        .unwrap_or_default();

    if verify {
        match crate::db::insert_member_from_manual(&data.db, user.id.0 as i64).await {
            Ok(()) => {
                let fresher = crate::db::get_member_by_id(&data.db, user.id.0 as i64)
                    .await?
                    .unwrap()
                    .fresher;
                let mut mm = m.member.clone().unwrap();
                crate::verify::apply_role(ctx, &mut mm, data.member).await?;
                if fresher {
                    crate::verify::apply_role(ctx, &mut mm, data.fresher).await?;
                }
                m.create_interaction_response(&ctx.http, |i| {
                    i.kind(serenity::InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|d| {
                            d.components(|c| c).embed(|e| {
                                e.thumbnail(m.user.avatar_url().unwrap_or(
                                    "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                                ))
                                .title("Member verified via manual")
                                .description(&user)
                                .field("Fresher", fresher, true)
                                .timestamp(serenity::Timestamp::now())
                            })
                        })
                })
                .await?
            }
            Err(e) => {
                eprintln!("Error: {e}");
                m.create_interaction_response(&ctx.http, |i| {
                    i.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d.content(format!("Failed to add user {user} to member database"))
                        })
                })
                .await?
            }
        }
    } else {
        println!("{} ({}) denied via manual", m.user.name, m.user.id);
        m.create_interaction_response(&ctx.http, |i| {
            i.kind(serenity::InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.components(|c| c).embed(|e| {
                        e.title("Member denied via manual")
                            .description(&user)
                            .thumbnail(user.avatar_url().unwrap_or(
                                "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                            ))
                            .timestamp(serenity::Timestamp::now())
                    })
                })
        })
        .await?
    }

    Ok(())
}
