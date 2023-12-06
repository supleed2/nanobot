use crate::{db, ACtx, Error, PendingMember};
use poise::serenity_prelude as serenity;
use poise::Modal;

/// Get the number of pending members in the pending table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn count_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    tracing::info!("{}", ctx.author().name);
    let count = db::count_pending(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the pending table"))
        .await?;
    Ok(())
}

/// Delete pending member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_pending(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    if db::delete_pending_by_id(&ctx.data().db, id.user.id.into()).await? {
        ctx.say(format!("Successfully deleted pending member info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete pending member info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all pending members in pending table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_all_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPending {
        #[name = "This will output the pending db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(ConfirmPending { confirm }) = ConfirmPending::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let pending = db::get_all_pending(&ctx.data().db).await?;
            match tokio::fs::write("pending.rs", format!("{pending:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending pending db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(&ctx.http(), vec!["pending.rs"], |cm| {
                            cm.content("File: pending db")
                        })
                        .await?;
                }
                Err(e) => {
                    tracing::error!("{e}");
                    ctx.say("Failed to create pending db file").await?;
                }
            }
            let _ = tokio::fs::remove_file("pending.rs").await;
            Ok(())
        } else {
            ctx.say("Skipping pending db output").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}

/// Get pending member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_pending(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_pending_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(p) => {
            ctx.say(format!("Pending info for {id}:\n```rust\n{p:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No pending entry found for {id}")).await?,
    };
    Ok(())
}

/// Manually add pending member to pending table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn add_pending(
    ctx: ACtx<'_>,
    id: serenity::Member,
    shortcode: String,
    realname: String,
) -> Result<(), Error> {
    tracing::info!(
        "{} {}, {shortcode}, {realname}",
        ctx.author().name,
        id.user.name,
    );
    db::insert_pending(
        &ctx.data().db,
        PendingMember {
            discord_id: id.user.id.into(),
            shortcode,
            realname,
        },
    )
    .await?;
    ctx.say(format!("Pending member added: {id}")).await?;
    Ok(())
}

/// Delete all pending members in pending table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_all_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPurgePending {
        #[name = "This will wipe the pending db"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(ConfirmPurgePending { confirm }) = ConfirmPurgePending::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let deleted = db::delete_all_pending(&ctx.data().db).await?;
            ctx.say(format!("Deleted {deleted} entries from the pending db"))
                .await?;
            Ok(())
        } else {
            ctx.say("Skipping pending db purge").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}
