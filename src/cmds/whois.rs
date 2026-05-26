use crate::{db, ACtx, Error};
use poise::{serenity_prelude as serenity, CreateReply, ReplyHandle};
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

/// Unreachable, used to create whois command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands("whois_by_id", "whois_by_nickname", "whois_by_realname")
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
