use crate::{db, ACtx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
// use std::fmt::Write;

/// Unreachable, used to create whois command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands("whois_by_id", "whois_by_nickname", "whois_by_realname")
)]
pub(crate) async fn whois(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// (Public) Get member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn whois_by_id(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_member_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(m) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("{id}: {}", m.nickname))
                    .ephemeral(true),
            )
            .await?
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!("No member entry found for {id}"))
                    .ephemeral(true),
            )
            .await?
        }
    };
    Ok(())
}

/// (Public) Get member info by Nickname (Exact)
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn whois_by_nickname(ctx: ACtx<'_>, nickname: String) -> Result<(), Error> {
    tracing::info!("{} {nickname}", ctx.author().name);
    if let Some(m) = db::get_member_by_nickname(&ctx.data().db, &nickname).await? {
        ctx.send(
            CreateReply::default()
                .content(format!("{nickname}: <@{}>", m.discord_id))
                .ephemeral(true),
        )
        .await?;
    } else {
        // let members = db::get_member_by_nickname_fuzzy(&ctx.data().db, &nickname, 3).await?;
        // if members.is_empty() {
        ctx.send(
            CreateReply::default()
                .content(format!("No member entry found for nickname {nickname}"))
                .ephemeral(true),
        )
        .await?;
        // } else {
        //     ctx.send(CreateReply::default().ephemeral(true).content(format!(
        //         "Possible matches for {nickname}: {}",
        //         members.iter().fold(String::new(), |mut s, g| {
        //             write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
        //             s
        //         })
        //     )))
        //     .await?;
        // }
    }
    Ok(())
}

/// (Public) Get member info by Real Name (Exact)
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn whois_by_realname(ctx: ACtx<'_>, realname: String) -> Result<(), Error> {
    tracing::info!("{} {realname}", ctx.author().name);
    if let Some(m) = db::get_member_by_realname(&ctx.data().db, &realname).await? {
        ctx.send(
            CreateReply::default()
                .content(format!("{realname}: <@{}>", m.discord_id))
                .ephemeral(true),
        )
        .await?;
    } else {
        // let members = db::get_member_by_realname_fuzzy(&ctx.data().db, &realname, 3).await?;
        // if members.is_empty() {
        ctx.send(
            CreateReply::default()
                .content(format!("No member entry found for realname {realname}"))
                .ephemeral(true),
        )
        .await?;
        // } else {
        //     ctx.send(CreateReply::default().ephemeral(true).content(format!(
        //         "Possible matches for {realname}: {}",
        //         members.iter().fold(String::new(), |mut s, g| {
        //             write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
        //             s
        //         })
        //     )))
        //     .await?;
        // }
    }
    Ok(())
}
