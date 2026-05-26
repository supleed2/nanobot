use crate::{db, ACtx, Error};
use poise::{
    serenity_prelude::{self as serenity, CreateEmbed, CreateMessage},
    CreateReply, ReplyHandle,
};
use std::fmt::Write as _;

trait EphemeralReply {
    async fn ereply(&self, c: impl Into<String>) -> Result<ReplyHandle<'_>, serenity::Error>;
}

impl EphemeralReply for ACtx<'_> {
    async fn ereply(&self, c: impl Into<String>) -> Result<ReplyHandle<'_>, serenity::Error> {
        let reply = CreateReply::default().ephemeral(true).content(c);
        self.send(reply).await
    }
}

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
        ctx.ereply(format!("Nick updated to {nickname}")).await?;
        let embed = CreateEmbed::new()
            .title("Nick updated")
            .thumbnail(u.face())
            .description(u.to_string())
            .field("Old Nick", old_nickname, true)
            .field("New Nick", nickname, true)
            .timestamp(serenity::Timestamp::now());
        let msg = CreateMessage::new().embed(embed);
        ctx.data().au_ch_id.send_message(ctx.http(), msg).await?;
    } else {
        ctx.ereply("Failed to update nick, please try again or message committee for help")
            .await?;
    }
    Ok(())
}

/// Unreachable, used to create whois command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands(
        "whois_by_id",
        "whois_by_nickname",
        "whois_by_realname",
        "whois_gaijin"
    )
)]
pub(crate) async fn whois(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// (Public) Find member by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn whois_by_id(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_member_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(m) => ctx.ereply(format!("{id}: {}", m.nickname)).await?,
        None => {
            ctx.ereply(format!("No member entry found for {id}"))
                .await?
        }
    };
    Ok(())
}

/// (Public) Find member by Nickname
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn whois_by_nickname(ctx: ACtx<'_>, nickname: String) -> Result<(), Error> {
    tracing::info!("{} {nickname}", ctx.author().name);
    if let Some(m) = db::get_member_by_nickname(&ctx.data().db, &nickname).await? {
        ctx.ereply(format!("{nickname}: <@{}>", m.discord_id))
            .await?;
    } else {
        let members = db::get_member_by_nickname_fuzzy(&ctx.data().db, &nickname, 3).await?;
        if members.is_empty() {
            ctx.ereply(format!("No member entry found for nickname {nickname}"))
                .await?;
        } else {
            let matches = members.iter().fold(String::new(), |mut s, g| {
                write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
                s
            });

            ctx.ereply(format!("Possible matches for {nickname}: {matches}"))
                .await?;
        }
    }
    Ok(())
}

/// (Public) Find member by Real Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn whois_by_realname(ctx: ACtx<'_>, realname: String) -> Result<(), Error> {
    tracing::info!("{} {realname}", ctx.author().name);
    if let Some(m) = db::get_member_by_realname(&ctx.data().db, &realname).await? {
        ctx.ereply(format!("{realname}: <@{}>", m.discord_id))
            .await?;
    } else {
        let members = db::get_member_by_realname_fuzzy(&ctx.data().db, &realname, 3).await?;
        if members.is_empty() {
            ctx.ereply(format!("No member entry found for realname {realname}"))
                .await?;
        } else {
            let matches = members.iter().fold(String::new(), |mut s, g| {
                write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
                s
            });

            ctx.ereply(format!("Possible matches for {realname}: {matches}"))
                .await?;
        }
    }
    Ok(())
}

/// (Public) Find gaijin by Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "gaijin")]
pub(crate) async fn whois_gaijin(ctx: ACtx<'_>, name: String) -> Result<(), Error> {
    tracing::info!("{} {name}", ctx.author().name);
    if let Some(m) = db::get_gaijin_by_name(&ctx.data().db, &name).await? {
        ctx.ereply(format!("{name}: <@{}>", m.discord_id)).await?;
    } else {
        let members = db::get_gaijin_by_name_fuzzy(&ctx.data().db, &name, 3).await?;
        if members.is_empty() {
            ctx.ereply(format!("No member entry found for name {name}"))
                .await?;
        } else {
            let matches = members.iter().fold(String::new(), |mut s, g| {
                write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
                s
            });

            ctx.ereply(format!("Possible matches for {name}: {matches}"))
                .await?;
        }
    }
    Ok(())
}
