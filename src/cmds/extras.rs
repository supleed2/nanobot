use crate::{db, verify, ACtx, Error, Gaijin};
use poise::{
    serenity_prelude::{self as serenity, CreateAttachment, CreateMessage},
    Modal,
};
use std::fmt::Write;

/// Get the number of entries in the gaijin table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn count_gaijin(ctx: ACtx<'_>) -> Result<(), Error> {
    tracing::info!("{}", ctx.author().name);
    let count = db::count_gaijin(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the gaijin table"))
        .await?;
    Ok(())
}

/// Delete gaijin info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn delete_gaijin(
    ctx: ACtx<'_>,
    mut id: serenity::Member,
    remove_roles: Option<bool>,
) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    if db::delete_gaijin_by_id(&ctx.data().db, id.user.id.into()).await? {
        if remove_roles.unwrap_or(true) {
            verify::remove_role(ctx.serenity_context(), &mut id, ctx.data().gaijin).await?;
        }
        ctx.say(format!("Successfully deleted gaijin info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete gaijin info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all gaijin in gaijin table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn get_all_gaijin(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct Confirm {
        #[name = "This will output the gaijin db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    tracing::info!("{}", ctx.author().name);

    if let Some(Confirm { confirm }) = Confirm::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let gaijin = db::get_all_gaijin(&ctx.data().db).await?;
            match tokio::fs::write("gaijin.rs", format!("{gaijin:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending gaijin db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(
                            &ctx.http(),
                            vec![CreateAttachment::path("gaijin.rs").await?],
                            CreateMessage::new().content("File: gaijin db"),
                        )
                        .await?;
                }
                Err(e) => {
                    tracing::error!("{e}");
                    ctx.say("Failed to create gaijin db file").await?;
                }
            }
            let _ = tokio::fs::remove_file("gaijin.rs").await;
            Ok(())
        } else {
            ctx.say("Skipping gaijin db output").await?;
            Ok(())
        }
    } else {
        ctx.say("Timed out").await?;
        Ok(())
    }
}

/// Unreachable, used to create get_gaijin command folder
#[allow(clippy::unused_async)]
#[poise::command(slash_command, subcommands("get_gaijin_by_id", "get_gaijin_by_name"))]
pub(crate) async fn get_gaijin(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// Get gaijin info by Discord ID
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn get_gaijin_by_id(ctx: ACtx<'_>, id: serenity::Member) -> Result<(), Error> {
    tracing::info!("{} {}", ctx.author().name, id.user.name);
    match db::get_gaijin_by_id(&ctx.data().db, id.user.id.into()).await? {
        Some(m) => {
            ctx.say(format!("Gaijin info for {id}:\n```rust\n{m:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No gaijin entry found for {id}")).await?,
    };
    Ok(())
}

/// Get gaijin info by Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn get_gaijin_by_name(ctx: ACtx<'_>, name: String) -> Result<(), Error> {
    tracing::info!("{} {name}", ctx.author().name);
    if let Some(g) = db::get_gaijin_by_name(&ctx.data().db, &name).await? {
        ctx.say(format!(
            "Gaijin info for name {name}:\n```rust\n{g:#?}\n```"
        ))
        .await?;
    } else {
        let gaijin = db::get_gaijin_by_name_fuzzy(&ctx.data().db, &name, 3).await?;
        if gaijin.is_empty() {
            ctx.say(format!("No entry found for name {name}")).await?;
        } else {
            ctx.say(format!(
                "Possible matches for {name}: {}",
                gaijin.iter().fold(String::new(), |mut s, g| {
                    write!(s, " <@{}>", g.discord_id).expect("String write! is infallible");
                    s
                })
            ))
            .await?;
        }
    }
    Ok(())
}

/// Add a gaijin to the gaijin table
#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub(crate) async fn add_gaijin(
    ctx: ACtx<'_>,
    mut id: serenity::Member,
    name: String,
    university: String,
) -> Result<(), Error> {
    tracing::info!(
        "{} {}, {name}, {university}",
        ctx.author().name,
        id.user.name,
    );
    db::insert_gaijin(
        &ctx.data().db,
        Gaijin {
            discord_id: id.user.id.into(),
            name,
            university,
        },
    )
    .await?;
    verify::apply_role(ctx.serenity_context(), &mut id, ctx.data().gaijin).await?;
    ctx.say(format!("Gaijin added: {id}")).await?;
    Ok(())
}

/// Unreachable, used to create edit_gaijin command folder
#[allow(clippy::unused_async)]
#[poise::command(
    slash_command,
    subcommands("edit_gaijin_name", "edit_gaijin_university")
)]
pub(crate) async fn edit_gaijin(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// Edit gaijin Name
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn edit_gaijin_name(
    ctx: ACtx<'_>,
    id: serenity::Member,
    name: String,
) -> Result<(), Error> {
    tracing::info!("{} {name}", ctx.author().name);
    if db::edit_gaijin_name(&ctx.data().db, id.user.id.into(), &name).await? {
        ctx.say(format!("{id} Name updated to {name}")).await?;
    } else {
        ctx.say(format!("Failed to update Name for {id}")).await?;
    }
    Ok(())
}

/// Edit gaijin University
#[tracing::instrument(skip_all)]
#[poise::command(slash_command, rename = "university")]
pub(crate) async fn edit_gaijin_university(
    ctx: ACtx<'_>,
    id: serenity::Member,
    university: String,
) -> Result<(), Error> {
    tracing::info!("{} {university}", ctx.author().name);
    if db::edit_gaijin_university(&ctx.data().db, id.user.id.into(), &university).await? {
        ctx.say(format!("{id} University updated to {university}"))
            .await?;
    } else {
        ctx.say(format!("Failed to update University for {id}"))
            .await?;
    }
    Ok(())
}
