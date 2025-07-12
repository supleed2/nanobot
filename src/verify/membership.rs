use crate::{db, ea, verify, Data, Error, Member};
use poise::serenity_prelude::{
    self as serenity, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage,
};
use poise::Modal;

const MEMBERSHIP_INTRO: &str = indoc::indoc! {"
    To use automatic verification via Membership:
    - Enter your Union order number (from this academic year)
    - Enter your Imperial shortcode
      - For Life members, your shortcode will be from when you were a student
      - For Associate members, this is your CID, in the format `AM-12345` or similar
    - Enter your preferred name for Nano whois commands
    - Your shortcode will then be connected to your Discord Account by Nano

    First, are you a fresher?
"};

#[tracing::instrument(skip_all)]
pub(crate) async fn membership_1(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    m.create_response(
        &ctx.http,
        CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .content(MEMBERSHIP_INTRO)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new("restart")
                        .style(serenity::ButtonStyle::Danger)
                        .emoji('🔙'),
                    CreateButton::new("membership_2f")
                        .style(serenity::ButtonStyle::Success)
                        .emoji('✅')
                        .label("Fresher"),
                    CreateButton::new("membership_2n")
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

#[tracing::instrument(skip_all)]
pub(crate) async fn membership_2(
    ctx: &serenity::Context,
    m: &serenity::ComponentInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    // Delete from pending if exists
    let _ = db::delete_pending_by_id(&data.db, m.user.id.into()).await;

    // Delete from manual if exists
    let _ = db::delete_manual_by_id(&data.db, m.user.id.into()).await;

    m.create_response(
        &ctx.http,
        Membership::create(
            None,
            if fresher {
                "membership_3f".to_string()
            } else {
                "membership_3n".to_string()
            },
        ),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all)]
pub(crate) async fn membership_3(
    ctx: &serenity::Context,
    m: &serenity::ModalInteraction,
    data: &Data,
    fresher: bool,
) -> Result<(), Error> {
    match Membership::parse(m.data.clone()) {
        Ok(Membership {
            order,
            shortcode,
            nickname,
        }) => {
            let members = match ea::get_members_list(&data.ea_key, &data.ea_url).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("{e}");
                    m.create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Sorry, getting membership data failed. Please try again")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                    return Ok(());
                }
            };
            let Some(member) = members.iter().find(|member| {
                ((member.login.is_empty() && member.cid == shortcode) || member.login == shortcode)
                    && member.order_no.to_string() == order
            }) else {
                let msg = "Sorry, your order was not found, please check the \
                        order number and that it is for your current year's membership";
                m.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(msg)
                            .ephemeral(true),
                    ),
                )
                .await?;
                return Ok(());
            };
            let realname = format!("{} {}", member.first_name, member.surname);
            if db::insert_member(
                &data.db,
                Member {
                    discord_id: m.user.id.into(),
                    shortcode,
                    nickname: nickname.clone(),
                    realname: realname.clone(),
                    fresher,
                },
            )
            .await
            .is_ok()
            {
                tracing::info!(
                    "{} ({}) added via membership{}",
                    m.user.name,
                    m.user.id,
                    if fresher { " (fresher)" } else { "" }
                );
                let mut mm = m.member.clone().unwrap();
                verify::apply_role(ctx, &mut mm, data.member).await?;
                if fresher {
                    verify::apply_role(ctx, &mut mm, data.fresher).await?;
                }
                m.create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content(if fresher {
                                "Congratulations, you have completed verification and now \
                                have access to the ICAS Discord and freshers thread"
                            } else {
                                "Congratulations, you have completed verification and now \
                                have access to the ICAS Discord"
                            })
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
                                .title("Member verified via membership")
                                .description(m.user.to_string())
                                .field("Fresher", fresher.to_string(), true)
                                .field("Nickname", nickname, true)
                                .field("Name", realname, true)
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
                return Ok(());
            }
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
