use crate::{Data, Error, verify};
use poise::serenity_prelude::{self as serenity, FullEvent};

pub(crate) async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::GuildMemberAddition { new_member } => {
            tracing::info!("Member joined: {}", new_member.user.name);
        }
        FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(m),
        } => {
            tracing::info!("Interaction: {} by {}", m.data.custom_id, m.user.name);
            match m.data.custom_id.as_str() {
                "register.global" | "unregister.global" | "register.guild" | "unregister.guild" => {
                }
                "info" => verify::info(ctx, m).await?,
                "start" => verify::start(ctx, m, data, true).await?,
                "restart" => verify::start(ctx, m, data, false).await?,
                "login_1" => verify::login_1(ctx, m).await?,
                "login_2" => verify::login_2(ctx, m, data).await?,
                "login_3" => verify::login_3(ctx, m).await?,
                "login_4f" => verify::login_4(ctx, m, true).await?,
                "login_4n" => verify::login_4(ctx, m, false).await?,
                "login_5f" => verify::login_5(ctx, m, true).await?,
                "login_5n" => verify::login_5(ctx, m, false).await?,
                "membership_1" => verify::membership_1(ctx, m).await?,
                "membership_2f" => verify::membership_2(ctx, m, data, true).await?,
                "membership_2n" => verify::membership_2(ctx, m, data, false).await?,
                "manual_1" => verify::manual_1(ctx, m).await?,
                "manual_2f" => verify::manual_2(ctx, m, data, true).await?,
                "manual_2n" => verify::manual_2(ctx, m, data, false).await?,
                id if id.starts_with("verify-") => verify::manual_4(ctx, m, data, id).await?,
                _ => {
                    tracing::info!("Unknown interaction, printing:\n{m:#?}");
                    verify::unknown(ctx, m).await?;
                }
            }
        }
        FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Modal(m),
        } => {
            tracing::info!("Modal submit: {} by {}", m.data.custom_id, m.user.name);
            match m.data.custom_id.as_str() {
                "login_6f" => verify::login_6(ctx, m, data, true).await?,
                "login_6n" => verify::login_6(ctx, m, data, false).await?,
                "membership_3f" => verify::membership_3(ctx, m, data, true).await?,
                "membership_3n" => verify::membership_3(ctx, m, data, false).await?,
                "manual_3f" => verify::manual_3(ctx, m, data, true).await?,
                "manual_3n" => verify::manual_3(ctx, m, data, false).await?,
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn init_tracing_subscriber() {
    use tracing_subscriber as ts;
    use ts::prelude::*;
    ts::registry()
        .with(
            ts::fmt::layer()
                .without_time()
                .with_filter(ts::EnvFilter::new(
                    "info,nano=info,shuttle=info,serenity=off",
                )),
        )
        .with(
            ts::fmt::layer()
                .without_time()
                .fmt_fields(ts::fmt::format::debug_fn(|w, f, v| {
                    if f.name() == "message" {
                        write!(w, "{v:?}")
                    } else {
                        write!(w, "")
                    }
                }))
                .with_filter(ts::EnvFilter::new("off,serenity=info")),
        )
        .init();
}
