use crate::{db, verify, ACtx, Error};
use poise::{
    serenity_prelude::{self as serenity, CreateEmbed, CreateMessage},
    CreateReply,
};

/// Update your nick according to nano (what shows up in `/whois`)
#[poise::command(slash_command)]
pub(crate) async fn nick(
    ctx: ACtx<'_>,
    #[min_length = 2]
    #[max_length = 32]
    nickname: String,
) -> Result<(), Error> {
    let u = ctx.author();
    let old_nickname = db::get_member_by_id(&ctx.data().db, u.id.into())
        .await?
        .map_or("<missing>".to_string(), |m| m.nickname);
    tracing::info!("{} {old_nickname} -> {nickname}", u.name);
    if db::edit_member_nickname(&ctx.data().db, u.id.into(), &nickname).await? {
        ctx.send(
            CreateReply::default()
                .content(format!("Nick updated to {nickname}"))
                .ephemeral(true),
        )
        .await?;
        ctx.data()
            .au_ch_id
            .send_message(
                ctx.http(),
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("Nick updated")
                        .thumbnail(u.face())
                        .description(u.to_string())
                        .field("Old Nick", old_nickname, true)
                        .field("New Nick", nickname, true)
                        .timestamp(serenity::Timestamp::now()),
                ),
            )
            .await?;
    } else {
        ctx.say("Failed to update nick, please try again or message committee for help")
            .await?;
    }
    Ok(())
}

/// Unreachable, used to create `edit_member` command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands(
        "edit_member_shortcode",
        "edit_member_nickname",
        "edit_member_realname",
        "edit_member_fresher",
    )
)]
pub(crate) async fn edit_member(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// Edit member Shortcode
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "shortcode")]
pub(crate) async fn edit_member_shortcode(
    ctx: ACtx<'_>,
    id: serenity::Member,
    shortcode: String,
) -> Result<(), Error> {
    tracing::info!("{} {shortcode}", ctx.author().name);
    if db::edit_member_shortcode(&ctx.data().db, id.user.id.into(), &shortcode).await? {
        ctx.say(format!("{id} Shortcode updated to {shortcode}"))
            .await?;
    } else {
        ctx.say(format!("Failed to update Shortcode for {id}"))
            .await?;
    }
    Ok(())
}

/// Edit member Nickname
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn edit_member_nickname(
    ctx: ACtx<'_>,
    id: serenity::Member,
    nickname: String,
) -> Result<(), Error> {
    tracing::info!("{} {nickname}", ctx.author().name);
    if db::edit_member_nickname(&ctx.data().db, id.user.id.into(), &nickname).await? {
        ctx.say(format!("{id} Nick updated to {nickname}")).await?;
    } else {
        ctx.say(format!("Failed to update Nick for {id}")).await?;
    }
    Ok(())
}

/// Edit member Real Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn edit_member_realname(
    ctx: ACtx<'_>,
    id: serenity::Member,
    realname: String,
) -> Result<(), Error> {
    tracing::info!("{} {realname}", ctx.author().name);
    if db::edit_member_realname(&ctx.data().db, id.user.id.into(), &realname).await? {
        ctx.say(format!("{id} Name updated to {realname}")).await?;
    } else {
        ctx.say(format!("Failed to update Name for {id}")).await?;
    }
    Ok(())
}

/// Edit member fresher status
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "fresher")]
pub(crate) async fn edit_member_fresher(
    ctx: ACtx<'_>,
    mut id: serenity::Member,
    fresher: bool,
) -> Result<(), Error> {
    tracing::info!("{} {} {fresher}", ctx.author().name, id.user.name);
    if db::edit_member_fresher(&ctx.data().db, id.user.id.into(), fresher).await? {
        if fresher {
            verify::apply_role(ctx.serenity_context(), &mut id, ctx.data().fresher).await?;
        } else {
            verify::remove_role(ctx.serenity_context(), &mut id, ctx.data().fresher).await?;
        }
        ctx.say(format!("{id} Fresher status updated to {fresher}"))
            .await?;
    } else {
        ctx.say(format!("Failed to update Fresher status for {id}"))
            .await?;
    }
    Ok(())
}

/// Set all members with no roles to non-member
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn refresh_non_members(ctx: ACtx<'_>) -> Result<(), Error> {
    use serenity::futures::StreamExt;
    tracing::info!("{}", ctx.author().name);
    ctx.defer().await?;
    let mut members = ctx.data().server.members_iter(ctx.http()).boxed();
    let mut cnt = 0;
    while let Some(Ok(m)) = members.next().await {
        if m.roles.is_empty() {
            m.add_role(ctx.http(), ctx.data().non_member).await?;
            cnt += 1;
        }
    }
    tracing::info!("{cnt} users given non-member role");
    ctx.say(format!(
        "{cnt} users given <@&{}> role",
        ctx.data().non_member
    ))
    .await?;
    Ok(())
}

/// Set all members to non-freshers
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn set_members_non_fresher(ctx: ACtx<'_>) -> Result<(), Error> {
    use serenity::futures::StreamExt;
    tracing::info!("{}", ctx.author().name);
    let updated = db::set_members_non_fresher(&ctx.data().db).await?;
    ctx.say(format!("{updated} updated to non-fresher, removing roles"))
        .await?;
    let mut members = ctx.data().server.members_iter(ctx.http()).boxed();
    while let Some(Ok(m)) = members.next().await {
        if m.roles.contains(&ctx.data().fresher) {
            let _ = m.remove_role(ctx.http(), ctx.data().fresher).await;
        }
    }
    ctx.say("Roles removed").await?;
    Ok(())
}
