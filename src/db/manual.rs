use crate::{Error, ManualMember};

/// Get count of entries in manual table
pub(crate) async fn count_manual(pool: &sqlx::PgPool) -> Result<i64, Error> {
    Ok(sqlx::query!("select count(*) as \"i64!\" from manual")
        .fetch_one(pool)
        .await?
        .i64)
}

/// Delete manual by Discord ID
pub(crate) async fn delete_manual_by_id(pool: &sqlx::PgPool, id: i64) -> Result<bool, Error> {
    let r = sqlx::query!("delete from manual where discord_id=$1", id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(r == 1)
}

/// Get all entries in manual table
pub(crate) async fn get_all_manual(pool: &sqlx::PgPool) -> Result<Vec<ManualMember>, Error> {
    Ok(sqlx::query_as!(ManualMember, "select * from manual")
        .fetch_all(pool)
        .await?)
}

/// Get manual entry by Discord ID
pub(crate) async fn get_manual_by_id(
    pool: &sqlx::PgPool,
    id: i64,
) -> Result<Option<ManualMember>, Error> {
    Ok(
        sqlx::query_as!(ManualMember, "select * from manual where discord_id=$1", id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Add manual entry to manual table
pub(crate) async fn insert_manual(pool: &sqlx::PgPool, m: ManualMember) -> Result<(), Error> {
    sqlx::query!(
        "insert into manual values ($1,$2,$3,$4,$5)",
        m.discord_id,
        m.shortcode.to_lowercase(),
        m.nickname,
        m.realname,
        m.fresher
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete all entries in manual table
pub(crate) async fn delete_all_manual(pool: &sqlx::PgPool) -> Result<u64, Error> {
    Ok(sqlx::query!("delete from manual")
        .execute(pool)
        .await?
        .rows_affected())
}
