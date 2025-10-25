use crate::{db, var, Error, Gaijin, ManualMember, Member, PendingMember};
use anyhow::Context as _;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};

pub(crate) fn router(pool: sqlx::PgPool) -> Result<axum::Router, Error> {
    let export_pool = pool.clone();
    let export_key = var!("EXPORT_KEY");
    let export_handler = |query| export(export_pool, query, export_key);

    let import_pool = pool.clone();
    let import_key = var!("IMPORT_KEY");
    let import_handler = |body| import(import_pool, body, import_key);

    let verify_pool = pool;
    let verify_key = var!("VERIFY_KEY");
    let verify_handler = |body| verify(verify_pool, body, verify_key);

    Ok(axum::Router::new()
        .route("/export", axum::routing::get(export_handler))
        .route("/import", axum::routing::post(import_handler))
        .route("/up", axum::routing::get(up))
        .route("/verify", axum::routing::post(verify_handler)))
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Key {
    key: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Db {
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

    let export = Db {
        pending,
        manual,
        members,
        extras,
    };

    (StatusCode::OK, Json(export)).into_response()
}

#[derive(serde::Deserialize)]
pub(crate) struct Import {
    db: Db,
    key: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn import(
    _pool: sqlx::PgPool,
    import: Option<Json<Import>>,
    expected_key: String,
) -> impl IntoResponse {
    let Some(Json(Import { db, key })) = import else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if key != expected_key {
        return StatusCode::NOT_FOUND.into_response();
    }

    format!(
        "Got {} pending, {} manual, {} members, {} extras",
        db.pending.len(),
        db.manual.len(),
        db.members.len(),
        db.extras.len()
    )
    .into_response()
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
    expected_key: String,
) -> impl IntoResponse {
    match payload {
        None => (StatusCode::BAD_REQUEST, "Invalid request body").into_response(),
        Some(Json(verify)) => {
            if verify.key == expected_key {
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
