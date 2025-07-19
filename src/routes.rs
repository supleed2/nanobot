use crate::{db, Gaijin, ManualMember, Member, PendingMember};
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Key {
    key: Option<String>,
}

#[derive(serde::Serialize)]
struct Export {
    pending: Vec<PendingMember>,
    manual: Vec<ManualMember>,
    members: Vec<Member>,
    extras: Vec<Gaijin>,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn export(
    pool: sqlx::PgPool,
    key: Query<Key>,
    expected_key: String,
) -> impl IntoResponse {
    if key.key.as_ref().is_none_or(|key| key != &expected_key) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let (Ok(pending), Ok(manual), Ok(members), Ok(extras)) = (
        db::get_all_pending(&pool).await,
        db::get_all_manual(&pool).await,
        db::get_all_members(&pool).await,
        db::get_all_gaijin(&pool).await,
    ) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "DB request failed").into_response();
    };

    let export = Export {
        pending,
        manual,
        members,
        extras,
    };

    (StatusCode::OK, Json(export)).into_response()
}

#[tracing::instrument(skip_all)]
pub(crate) async fn up() -> impl IntoResponse {
    (StatusCode::OK, "Nano is up!")
}

#[derive(serde::Deserialize)]
pub(crate) struct Verify {
    id: String,
    shortcode: String,
    fullname: String,
    key: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn verify(
    pool: sqlx::PgPool,
    payload: Option<Json<Verify>>,
    key: String,
) -> impl IntoResponse {
    match payload {
        None => (StatusCode::BAD_REQUEST, "Invalid request body").into_response(),
        Some(Json(verify)) => {
            if verify.key == key {
                let Ok(id) = verify.id.parse::<i64>() else {
                    return (StatusCode::BAD_REQUEST, "Invalid request body").into_response();
                };

                // Delete from pending if exists
                let _ = db::delete_pending_by_id(&pool, id).await;

                match db::insert_pending(
                    &pool,
                    PendingMember {
                        discord_id: id,
                        shortcode: verify.shortcode.clone(),
                        realname: verify.fullname.clone(),
                    },
                )
                .await
                {
                    Ok(()) => {
                        tracing::info!(
                            "ID {} added: {}, {}",
                            id,
                            verify.shortcode,
                            verify.fullname
                        );
                        (StatusCode::OK, "Member added to `pending` database").into_response()
                    }
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
                }
            } else {
                (StatusCode::UNAUTHORIZED, "Auth required").into_response()
            }
        }
    }
}
