use crate::{db, verify, ACtx, Error, Member};
use poise::serenity_prelude::{self as serenity, CreateAttachment, CreateMessage};
use poise::Modal;

/// Get the number of members in the members table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn count_members(ctx: ACtx<'_>) -> Result<(), Error> {
    tracing::info!("{}", ctx.author().name);
    let count = db::count_members(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the members table"))
        .await?;
    Ok(())
}

/// Delete member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_member(
    ctx: ACtx<'_>,
    mut id: serenity::Member,
    remove_roles: Option<bool>,
) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    if db::delete_member_by_id(&ctx.data().db, id.user.id.into()).await? {
        if remove_roles.unwrap_or(true) {
            verify::remove_role(ctx.serenity_context(), &mut id, ctx.data().member).await?;
            verify::remove_role(ctx.serenity_context(), &mut id, ctx.data().fresher).await?;
        }
        ctx.say(format!("Successfully deleted member info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete member info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all members in members table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_all_members(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct Confirm {
        #[name = "This will output the members db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(Confirm { confirm }) = Confirm::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let members = db::get_all_members(&ctx.data().db).await?;
            match tokio::fs::write("members.rs", format!("{members:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending members db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(
                            &ctx.http(),
                            vec![CreateAttachment::path("members.rs").await?],
                            CreateMessage::new().content("File: members db"),
                        )
                        .await?;
                }
                Err(e) => {
                    tracing::error!("{e}");
                    ctx.say("Failed to create members db file").await?;
                }
            }
            let _ = tokio::fs::remove_file("members.rs").await;
            Ok(())
        } else {
            ctx.say("Skipping members db output").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}

/// Unreachable, used to create get_member command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands(
        "get_member_by_id",
        "get_member_by_shortcode",
        "get_member_by_nickname",
        "get_member_by_realname",
    )
)]
pub(crate) async fn get_member(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// Get member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn get_member_by_id(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_member_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(m) => {
            ctx.say(format!("Member info for {id}:\n```rust\n{m:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No member entry found for {id}")).await?,
    };
    Ok(())
}

/// Get member info by Shortcode
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "shortcode")]
pub(crate) async fn get_member_by_shortcode(ctx: ACtx<'_>, shortcode: String) -> Result<(), Error> {
    tracing::info!("{} {shortcode}", ctx.author().name);
    match db::get_member_by_shortcode(&ctx.data().db, &shortcode).await? {
        Some(m) => {
            ctx.say(format!(
                "Member info for shortcode {shortcode}:\n```rust\n{m:#?}\n```"
            ))
            .await?
        }
        None => {
            ctx.say(format!("No entry found for shortcode {shortcode}"))
                .await?
        }
    };
    Ok(())
}

/// Get member info by Nickname
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn get_member_by_nickname(ctx: ACtx<'_>, nickname: String) -> Result<(), Error> {
    tracing::info!("{} {nickname}", ctx.author().name);
    match db::get_member_by_nickname(&ctx.data().db, &nickname).await? {
        Some(m) => {
            ctx.say(format!(
                "Member info for nickname {nickname}:\n```rust\n{m:#?}\n```"
            ))
            .await?
        }
        None => {
            ctx.say(format!("No entry found for nickname {nickname}"))
                .await?
        }
    };
    Ok(())
}

/// Get member info by Real Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn get_member_by_realname(ctx: ACtx<'_>, realname: String) -> Result<(), Error> {
    tracing::info!("{} {realname}", ctx.author().name);
    match db::get_member_by_realname(&ctx.data().db, &realname).await? {
        Some(m) => {
            ctx.say(format!(
                "Member info for realname {realname}:\n```rust\n{m:#?}\n```"
            ))
            .await?
        }
        None => {
            ctx.say(format!("No entry found for realname {realname}"))
                .await?
        }
    };
    Ok(())
}

/// Add a member to the members table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn add_member(
    ctx: ACtx<'_>,
    mut id: serenity::Member,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
) -> Result<(), Error> {
    tracing::info!(
        "{} {}, {shortcode}, {realname}, {nickname}",
        ctx.author().name,
        id.user.name,
    );
    db::insert_member(
        &ctx.data().db,
        Member {
            discord_id: id.user.id.into(),
            shortcode,
            nickname,
            realname,
            fresher,
        },
    )
    .await?;
    verify::remove_role(ctx.serenity_context(), &mut id, ctx.data().non_member).await?;
    verify::apply_role(ctx.serenity_context(), &mut id, ctx.data().member).await?;
    if fresher {
        verify::apply_role(ctx.serenity_context(), &mut id, ctx.data().fresher).await?;
    }
    ctx.say(format!("Member added: {id}")).await?;
    Ok(())
}

/// Manually add member to members table from pending table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn insert_member_from_pending(
    ctx: ACtx<'_>,
    id: serenity::Member,
    nickname: String,
    fresher: bool,
) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::insert_member_from_pending(&ctx.data().db, id.user.id.into(), &nickname, fresher)
        .await
    {
        Ok(_) => {
            ctx.say(format!("Member moved from pending to members table: {id}"))
                .await?
        }
        Err(e) => ctx.say(format!("Error: {e}")).await?,
    };
    Ok(())
}

/// Manually add member to members table from manual table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn insert_member_from_manual(
    ctx: ACtx<'_>,
    id: serenity::Member,
) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::insert_member_from_manual(&ctx.data().db, id.user.id.into()).await {
        Ok(_) => {
            ctx.say(format!("Member moved from manual to members table: {id}"))
                .await?
        }
        Err(e) => ctx.say(format!("Error: {e}")).await?,
    };
    Ok(())
}
