use crate::{db, ACtx, Error, Fresher, ManualMember};
use poise::serenity_prelude::{self as serenity, CreateAttachment, CreateMessage};
use poise::Modal;

/// Get the number of manual members in the manual table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn count_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    tracing::info!("{}", ctx.author().name);
    let count = db::count_manual(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the manual table"))
        .await?;
    Ok(())
}

/// Delete manual member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_manual(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    if db::delete_manual_by_id(&ctx.data().db, id.user.id.into()).await? {
        ctx.say(format!("Successfully deleted manual member info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete manual member info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all manual members in manual table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_all_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmManual {
        #[name = "This will output the manual db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(ConfirmManual { confirm }) = ConfirmManual::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let manual = db::get_all_manual(&ctx.data().db).await?;
            match tokio::fs::write("manual.rs", format!("{manual:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending manual db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(
                            &ctx.http(),
                            vec![CreateAttachment::path("manual.rs").await?],
                            CreateMessage::new().content("File: manual db"),
                        )
                        .await?;
                }
                Err(e) => {
                    tracing::error!("{e}");
                    ctx.say("Failed to create manual db file").await?;
                }
            }
            let _ = tokio::fs::remove_file("manual.rs").await;
            Ok(())
        } else {
            ctx.say("Skipping manual db output").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}

/// Get manual member info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_manual(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_manual_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(m) => {
            ctx.say(format!("Manual info for {id}:\n```rust\n{m:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No manual entry found for {id}")).await?,
    };
    Ok(())
}

/// Manually add manual member to manual table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn add_manual(
    ctx: ACtx<'_>,
    id: serenity::Member,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: Fresher,
) -> Result<(), Error> {
    tracing::info!(
        "{} {}, {shortcode}, {realname}, {nickname}",
        ctx.author().name,
        id.user.name,
    );
    db::insert_manual(
        &ctx.data().db,
        ManualMember {
            discord_id: id.user.id.into(),
            shortcode,
            nickname,
            realname,
            fresher,
        },
    )
    .await?;
    ctx.say(format!("Manual member added: {id}")).await?;
    Ok(())
}

/// Delete all manual members in manual table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_all_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPurgeManual {
        #[name = "This will wipe the manual db"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(ConfirmPurgeManual { confirm }) = ConfirmPurgeManual::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let deleted = db::delete_all_manual(&ctx.data().db).await?;
            ctx.say(format!("Deleted {deleted} entries from the manual db"))
                .await?;
            Ok(())
        } else {
            ctx.say("Skipping manual db purge").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}
