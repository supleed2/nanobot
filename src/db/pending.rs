use crate::{Error, PendingMember};

/// Get count of entries in pending table
pub(crate) async fn count_pending(pool: &sqlx::SqlitePool) -> Result<i64, Error> {
    Ok(sqlx::query!("select count(*) as \"i64!\" from pending")
        .fetch_one(pool)
        .await?
        .i64)
}

/// Delete pending by Discord ID
pub(crate) async fn delete_pending_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<bool, Error> {
    let r = sqlx::query!("delete from pending where discord_id=$1", id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(r == 1)
}

/// Get all entries in pending table
pub(crate) async fn get_all_pending(pool: &sqlx::SqlitePool) -> Result<Vec<PendingMember>, Error> {
    Ok(sqlx::query_as!(PendingMember, "select * from pending")
        .fetch_all(pool)
        .await?)
}

/// Get pending entry by Discord ID
pub(crate) async fn get_pending_by_id(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<Option<PendingMember>, Error> {
    Ok(sqlx::query_as!(
        PendingMember,
        "select * from pending where discord_id=$1",
        id
    )
    .fetch_optional(pool)
    .await?)
}

/// Add pending entry to pending table
pub(crate) async fn insert_pending(pool: &sqlx::SqlitePool, p: PendingMember) -> Result<(), Error> {
    let shortcode = p.shortcode.to_lowercase();
    sqlx::query!(
        "insert into pending values ($1, $2, $3)",
        p.discord_id,
        shortcode,
        p.realname
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete all entries in pending table
pub(crate) async fn delete_all_pending(pool: &sqlx::SqlitePool) -> Result<u64, Error> {
    Ok(sqlx::query!("delete from pending")
        .execute(pool)
        .await?
        .rows_affected())
}
