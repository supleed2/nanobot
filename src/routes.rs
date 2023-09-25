use crate::PendingMember;
use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct Verify {
    id: String,
    shortcode: String,
    fullname: String,
    key: String,
}

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
                let _ = crate::db::delete_pending_by_id(&pool, id).await;

                match crate::db::insert_pending(
                    &pool,
                    PendingMember {
                        discord_id: id,
                        shortcode: verify.shortcode.clone(),
                        realname: verify.fullname.clone(),
                    },
                )
                .await
                {
                    Ok(_) => {
                        println!("ID {} added: {}, {}", id, verify.shortcode, verify.fullname);
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
