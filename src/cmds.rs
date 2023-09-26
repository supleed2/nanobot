use crate::{db, ACtx, Data, Error, ManualMember, Member, PendingMember};
use poise::serenity_prelude as serenity;
use poise::Modal;

/// Buttons to (de-)register application commands globally or by guild
#[poise::command(prefix_command, owners_only)]
pub(crate) async fn cmds(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    println!("Cmd: ({}) cmds", ctx.author().name);
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Send (customisable) verification introduction message in specified channel
#[poise::command(slash_command)]
pub(crate) async fn setup(
    ctx: ACtx<'_>,
    #[description = "Channel to send verification introduction message in"]
    #[channel_types("Text", "News")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    #[derive(Modal)]
    struct Setup {
        #[name = "Contents of the verification intro message"]
        #[placeholder = "Welcome... Click the \"Begin\" button below to start verification"]
        #[paragraph]
        #[max_length = 500]
        message: String,
        #[name = "Emoji for the start button"]
        #[placeholder = "🚀"]
        #[max_length = 4]
        emoji: Option<String>,
        #[name = "Label for the start button"]
        #[placeholder = "Begin"]
        #[max_length = 80]
        text: Option<String>,
    }

    println!("Cmd: ({}) setup", ctx.author().name);

    if let Some(Setup {
        message,
        emoji,
        text,
    }) = Setup::execute(ctx).await?
    {
        ctx.say(format!("Sending intro message in {channel}"))
            .await?;
        let emoji = emoji.unwrap_or_default().chars().next().unwrap_or('🚀');
        channel
            .send_message(ctx.http(), |m| {
                m.content(message).components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(serenity::ButtonStyle::Secondary)
                                .emoji('📖')
                                .label("More info")
                                .custom_id("info")
                        })
                        .create_button(|b| {
                            b.style(serenity::ButtonStyle::Primary)
                                .emoji(emoji)
                                .label(text.unwrap_or("Begin".to_string()))
                                .custom_id("start")
                        })
                    })
                })
            })
            .await?;
    } else {
        ctx.say("Modal timed out, try again...").await?;
    }
    Ok(())
}

/// Get the number of members in the members table
#[poise::command(slash_command)]
pub(crate) async fn count_members(ctx: ACtx<'_>) -> Result<(), Error> {
    println!("Cmd: ({}) count_members", ctx.author().name);
    let count = db::count_members(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the members table"))
        .await?;
    Ok(())
}

/// Delete member info by Discord ID
#[poise::command(slash_command)]
pub(crate) async fn delete_member(
    ctx: ACtx<'_>,
    id: serenity::Member,
    remove_roles: Option<bool>,
) -> Result<(), Error> {
    println!("Cmd: ({}) delete_member {id}", ctx.author().name);
    if db::delete_member_by_id(&ctx.data().db, id.user.id.into()).await? {
        if remove_roles.unwrap_or(true) {
            let mut m = id.clone();
            crate::verify::remove_role(ctx.serenity_context(), &mut m, ctx.data().member).await?;
            crate::verify::remove_role(ctx.serenity_context(), &mut m, ctx.data().fresher).await?;
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
#[poise::command(slash_command)]
pub(crate) async fn get_all_members(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct Confirm {
        #[name = "This will output the members db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    println!("Cmd: ({}) get_all_members", ctx.author().name);

    if let Some(Confirm { confirm }) = Confirm::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let members = db::get_all_members(&ctx.data().db).await?;
            match tokio::fs::write("members.rs", format!("{members:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending members db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(&ctx.http(), vec!["members.rs"], |cm| {
                            cm.content("File: members db")
                        })
                        .await?;
                }
                Err(e) => {
                    eprintln!("Error: {e}");
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
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn get_member_by_id(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) get_member_by_id {id}", ctx.author().name);
    match db::get_member_by_id(&ctx.data().db, id.id.into()).await? {
        Some(m) => {
            ctx.say(format!("Member info for {id}:\n```rust\n{m:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No member entry found for {id}")).await?,
    };
    Ok(())
}

/// Get member info by Shortcode
#[poise::command(slash_command, rename = "shortcode")]
pub(crate) async fn get_member_by_shortcode(ctx: ACtx<'_>, shortcode: String) -> Result<(), Error> {
    println!(
        "Cmd: ({}) get_member_by_shortcode {shortcode}",
        ctx.author().name
    );
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
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn get_member_by_nickname(ctx: ACtx<'_>, nickname: String) -> Result<(), Error> {
    println!(
        "Cmd: ({}) get_member_by_nickname {nickname}",
        ctx.author().name
    );
    match db::get_member_by_nickname(&ctx.data().db, &nickname).await? {
        Some(m) => {
            ctx.say(format!(
                "Member info for nickname {nickname}:\n```rust\n{m:#?}\n```"
            ))
            .await?
        }
        None => {
            ctx.say(format!("No entry found for nickname {nickname}",))
                .await?
        }
    };
    Ok(())
}

/// Get member info by Real Name
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn get_member_by_realname(ctx: ACtx<'_>, realname: String) -> Result<(), Error> {
    println!(
        "Cmd: ({}) get_member_by_realname {realname}",
        ctx.author().name
    );
    match db::get_member_by_realname(&ctx.data().db, &realname).await? {
        Some(m) => {
            ctx.say(format!(
                "Member info for realname {realname}:\n```rust\n{m:#?}\n```"
            ))
            .await?
        }
        None => {
            ctx.say(format!("No entry found for realname {realname}",))
                .await?
        }
    };
    Ok(())
}

/// Add a member to the members table
#[poise::command(slash_command)]
pub(crate) async fn add_member(
    ctx: ACtx<'_>,
    id: serenity::Member,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) add_member {id}, {shortcode}, {realname}, {nickname}",
        ctx.author().name,
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
    let mut m = id.clone();
    crate::verify::apply_role(ctx.serenity_context(), &mut m, ctx.data().member).await?;
    if fresher {
        crate::verify::apply_role(ctx.serenity_context(), &mut m, ctx.data().fresher).await?;
    }
    ctx.say(format!("Member added: {id}")).await?;
    Ok(())
}

/// Manually add member to members table from pending table
#[poise::command(slash_command)]
pub(crate) async fn insert_member_from_pending(
    ctx: ACtx<'_>,
    id: serenity::User,
    nickname: String,
    fresher: bool,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) insert_member_from_pending {id}",
        ctx.author().name
    );
    match db::insert_member_from_pending(&ctx.data().db, id.id.into(), &nickname, fresher).await {
        Ok(()) => {
            ctx.say(format!("Member moved from pending to members table: {id}"))
                .await?
        }
        Err(e) => ctx.say(format!("Error: {e}")).await?,
    };
    Ok(())
}

/// Manually add member to members table from manual table
#[poise::command(slash_command)]
pub(crate) async fn insert_member_from_manual(
    ctx: ACtx<'_>,
    id: serenity::User,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) insert_member_from_manual {id}",
        ctx.author().name
    );
    match db::insert_member_from_manual(&ctx.data().db, id.id.into()).await {
        Ok(()) => {
            ctx.say(format!("Member moved from manual to members table: {id}"))
                .await?
        }
        Err(e) => ctx.say(format!("Error: {e}")).await?,
    };
    Ok(())
}

/// Unreachable, used to create edit_member command folder
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
#[poise::command(slash_command, rename = "shortcode")]
pub(crate) async fn edit_member_shortcode(
    ctx: ACtx<'_>,
    id: serenity::Member,
    shortcode: String,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) edit_member_shortcode {shortcode}",
        ctx.author().name
    );
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
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn edit_member_nickname(
    ctx: ACtx<'_>,
    id: serenity::Member,
    nickname: String,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) edit_member_nickname {nickname}",
        ctx.author().name
    );
    if db::edit_member_nickname(&ctx.data().db, id.user.id.into(), &nickname).await? {
        ctx.say(format!("{id} Nick updated to {nickname}")).await?;
    } else {
        ctx.say(format!("Failed to update Nick for {id}")).await?;
    }
    Ok(())
}

/// Edit member Real Name
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn edit_member_realname(
    ctx: ACtx<'_>,
    id: serenity::Member,
    realname: String,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) edit_member_realname {realname}",
        ctx.author().name
    );
    if db::edit_member_realname(&ctx.data().db, id.user.id.into(), &realname).await? {
        ctx.say(format!("{id} Name updated to {realname}")).await?;
    } else {
        ctx.say(format!("Failed to update Name for {id}")).await?;
    }
    Ok(())
}

/// Edit member fresher status
#[poise::command(slash_command, rename = "fresher")]
pub(crate) async fn edit_member_fresher(
    ctx: ACtx<'_>,
    id: serenity::Member,
    fresher: bool,
) -> Result<(), Error> {
    println!("Cmd: ({}) edit_member_fresher {fresher}", ctx.author().name);
    if db::edit_member_fresher(&ctx.data().db, id.user.id.into(), fresher).await? {
        ctx.say(format!("{id} Fresher status updated to {fresher}"))
            .await?;
    } else {
        ctx.say(format!("Failed to update Fresher status for {id}"))
            .await?;
    }
    Ok(())
}
/// Get the number of pending members in the pending table
#[poise::command(slash_command)]
pub(crate) async fn count_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    println!("Cmd: ({}) count_pending", ctx.author().name);
    let count = db::count_pending(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the pending table"))
        .await?;
    Ok(())
}

/// Delete pending member info by Discord ID
#[poise::command(slash_command)]
pub(crate) async fn delete_pending(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) delete_pending {id}", ctx.author().name);
    if db::delete_pending_by_id(&ctx.data().db, id.id.into()).await? {
        ctx.say(format!("Successfully deleted pending member info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete pending member info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all pending members in pending table
#[poise::command(slash_command)]
pub(crate) async fn get_all_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPending {
        #[name = "This will output the pending db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    println!("Cmd: ({}) get_all_pending", ctx.author().name);

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
                    eprintln!("Error: {e}");
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
#[poise::command(slash_command)]
pub(crate) async fn get_pending(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) get_pending {id}", ctx.author().name);
    match db::get_pending_by_id(&ctx.data().db, id.id.into()).await? {
        Some(p) => {
            ctx.say(format!("Pending info for {id}:\n```rust\n{p:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No pending entry found for {id}")).await?,
    };
    Ok(())
}

/// Manually add pending member to pending table
#[poise::command(slash_command)]
pub(crate) async fn add_pending(
    ctx: ACtx<'_>,
    id: serenity::User,
    shortcode: String,
    realname: String,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) add_pending {id}, {shortcode}, {realname}",
        ctx.author().name
    );
    db::insert_pending(
        &ctx.data().db,
        PendingMember {
            discord_id: id.id.into(),
            shortcode,
            realname,
        },
    )
    .await?;
    ctx.say(format!("Pending member added: {id}")).await?;
    Ok(())
}

/// Delete all pending members in pending table
#[poise::command(slash_command)]
pub(crate) async fn delete_all_pending(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPurgePending {
        #[name = "This will wipe the pending db"]
        #[placeholder = "yes"]
        confirm: String,
    }

    println!("Cmd: ({}) delete_all_pending", ctx.author().name);

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

/// Get the number of manual members in the manual table
#[poise::command(slash_command)]
pub(crate) async fn count_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    println!("Cmd: ({}) count_manual", ctx.author().name);
    let count = db::count_manual(&ctx.data().db).await?;
    ctx.say(format!("There are {count} entries in the manual table"))
        .await?;
    Ok(())
}

/// Delete manual member info by Discord ID
#[poise::command(slash_command)]
pub(crate) async fn delete_manual(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) delete_manual {id}", ctx.author().name);
    if db::delete_manual_by_id(&ctx.data().db, id.id.into()).await? {
        ctx.say(format!("Successfully deleted manual member info for {id}"))
            .await?
    } else {
        ctx.say(format!("Failed to delete manual member info for {id}"))
            .await?
    };
    Ok(())
}

/// Print all manual members in manual table
#[poise::command(slash_command)]
pub(crate) async fn get_all_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmManual {
        #[name = "This will output the manual db as text"]
        #[placeholder = "yes"]
        confirm: String,
    }

    println!("Cmd: ({}) get_all_manual", ctx.author().name);

    if let Some(ConfirmManual { confirm }) = ConfirmManual::execute(ctx).await? {
        if confirm.to_lowercase().contains("yes") {
            let manual = db::get_all_manual(&ctx.data().db).await?;
            match tokio::fs::write("manual.rs", format!("{manual:#?}")).await {
                Ok(()) => {
                    ctx.say("Sending manual db data in followup message")
                        .await?;
                    ctx.channel_id()
                        .send_files(&ctx.http(), vec!["manual.rs"], |cm| {
                            cm.content("File: manual db")
                        })
                        .await?;
                }
                Err(e) => {
                    eprintln!("Error: {e}");
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
#[poise::command(slash_command)]
pub(crate) async fn get_manual(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) get_manual {id}", ctx.author().name);
    match db::get_manual_by_id(&ctx.data().db, id.id.into()).await? {
        Some(m) => {
            ctx.say(format!("Manual info for {id}:\n```rust\n{m:#?}\n```"))
                .await?
        }
        None => ctx.say(format!("No manual entry found for {id}")).await?,
    };
    Ok(())
}

/// Manually add manual member to manual table
#[poise::command(slash_command)]
pub(crate) async fn add_manual(
    ctx: ACtx<'_>,
    id: serenity::User,
    shortcode: String,
    nickname: String,
    realname: String,
    fresher: bool,
) -> Result<(), Error> {
    println!(
        "Cmd: ({}) add_manual {id}, {shortcode}, {realname}, {nickname}",
        ctx.author().name
    );
    db::insert_manual(
        &ctx.data().db,
        ManualMember {
            discord_id: id.id.into(),
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
#[poise::command(slash_command)]
pub(crate) async fn delete_all_manual(ctx: ACtx<'_>) -> Result<(), Error> {
    #[derive(Modal)]
    struct ConfirmPurgeManual {
        #[name = "This will wipe the manual db"]
        #[placeholder = "yes"]
        confirm: String,
    }

    println!("Cmd: ({}) delete_all_manual", ctx.author().name);

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

/// Unreachable, used to create whois command folder
#[poise::command(
    slash_command,
    subcommands("whois_by_id", "whois_by_nickname", "whois_by_realname")
)]
pub(crate) async fn whois(_ctx: ACtx<'_>) -> Result<(), Error> {
    unreachable!()
}

/// (Public) Get member info by Discord ID
#[poise::command(slash_command, rename = "id")]
pub(crate) async fn whois_by_id(ctx: ACtx<'_>, id: serenity::User) -> Result<(), Error> {
    println!("Cmd: ({}) whois_by_id {id}", ctx.author().name);
    match db::get_member_by_id(&ctx.data().db, id.id.into()).await? {
        Some(m) => {
            ctx.send(|c| c.content(format!("{id}: {}", m.nickname)).ephemeral(true))
                .await?
        }
        None => ctx.say(format!("No member entry found for {id}")).await?,
    };
    Ok(())
}

/// (Public) Get member info by Nickname (Exact)
#[poise::command(slash_command, rename = "nick")]
pub(crate) async fn whois_by_nickname(ctx: ACtx<'_>, nickname: String) -> Result<(), Error> {
    println!("Cmd: ({}) whois_by_nickname {nickname}", ctx.author().name);
    if let Some(m) = db::get_member_by_nickname(&ctx.data().db, &nickname).await? {
        ctx.send(|c| {
            c.content(format!("{nickname}: <@{}>", m.discord_id))
                .ephemeral(true)
        })
        .await?;
    } else {
        let members = db::get_member_by_nickname_fuzzy(&ctx.data().db, &nickname, 3).await?;
        if members.is_empty() {
            ctx.send(|c| {
                c.content(format!("No member entry found for nickname {nickname}"))
                    .ephemeral(true)
            })
            .await?;
        } else {
            ctx.send(|c| {
                c.ephemeral(true).content(format!(
                    "Possible matches for {nickname}: {}",
                    members
                        .iter()
                        .map(|m| format!(" <@{}>", m.discord_id))
                        .collect::<String>()
                ))
            })
            .await?;
        }
    }
    Ok(())
}

/// (Public) Get member info by Real Name (Exact)
#[poise::command(slash_command, rename = "name")]
pub(crate) async fn whois_by_realname(ctx: ACtx<'_>, realname: String) -> Result<(), Error> {
    println!("Cmd: ({}) whois_by_realname {realname}", ctx.author().name);
    if let Some(m) = db::get_member_by_realname(&ctx.data().db, &realname).await? {
        ctx.send(|c| {
            c.content(format!("{realname}: <@{}>", m.discord_id))
                .ephemeral(true)
        })
        .await?;
    } else {
        let members = db::get_member_by_realname_fuzzy(&ctx.data().db, &realname, 3).await?;
        if members.is_empty() {
            ctx.send(|c| {
                c.content(format!("No member entry found for realname {realname}"))
                    .ephemeral(true)
            })
            .await?;
        } else {
            ctx.send(|c| {
                c.ephemeral(true).content(format!(
                    "Possible matches for {realname}: {}",
                    members
                        .iter()
                        .map(|m| format!(" <@{}>", m.discord_id))
                        .collect::<String>()
                ))
            })
            .await?;
        }
    };
    Ok(())
}
