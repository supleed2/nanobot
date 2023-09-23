use crate::{Error, ManualMember, Member, PendingMember};

/// Get count of entries in members table
pub(crate) async fn count_members(pool: &sqlx::PgPool) -> Result<i64, Error> {
    Ok(sqlx::query!("select count(*) as \"i64!\" from members")
        .fetch_one(pool)
        .await?
        .i64)
}

/// Delete member by Discord ID
pub(crate) async fn delete_member_by_id(pool: &sqlx::PgPool, id: i64) -> Result<bool, Error> {
    let r = sqlx::query!("delete from members where discord_id=$1", id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(r == 1)
}

/// Get all entries in members table
pub(crate) async fn get_all_members(pool: &sqlx::PgPool) -> Result<Vec<Member>, Error> {
    Ok(sqlx::query_as!(Member, "select * from members")
        .fetch_all(pool)
        .await?)
}

/// Get member entry by Discord ID
pub(crate) async fn get_member_by_id(
    pool: &sqlx::PgPool,
    id: i64,
) -> Result<Option<Member>, Error> {
    Ok(
        sqlx::query_as!(Member, "select * from members where discord_id=$1", id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Get member entry by Shortcode
pub(crate) async fn get_member_by_shortcode(
    pool: &sqlx::PgPool,
    shortcode: &str,
) -> Result<Option<Member>, Error> {
    Ok(sqlx::query_as!(
        Member,
        "select * from members where shortcode=$1",
        shortcode
    )
    .fetch_optional(pool)
    .await?)
}

/// Get member entry by Nickname
pub(crate) async fn get_member_by_nickname(
    pool: &sqlx::PgPool,
    nickname: &str,
) -> Result<Option<Member>, Error> {
    Ok(
        sqlx::query_as!(Member, "select * from members where nickname=$1", nickname)
            .fetch_optional(pool)
            .await?,
    )
}

/// Get member entry by Real Name
pub(crate) async fn get_member_by_realname(
    pool: &sqlx::PgPool,
    realname: &str,
) -> Result<Option<Member>, Error> {
    Ok(
        sqlx::query_as!(Member, "select * from members where realname=$1", realname)
            .fetch_optional(pool)
            .await?,
    )
}

/// Add member entry to members table
pub(crate) async fn insert_member(pool: &sqlx::PgPool, m: Member) -> Result<(), Error> {
    sqlx::query!(
        "insert into members values ($1, $2, $3, $4, $5)",
        m.discord_id,
        m.shortcode,
        m.nickname,
        m.realname,
        m.fresher
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Add member entry to members table from pending table
pub(crate) async fn insert_member_from_pending(
    pool: &sqlx::PgPool,
    id: i64,
    nickname: &str,
    fresher: bool,
) -> Result<(), Error> {
    let p = sqlx::query_as!(
        PendingMember,
        "delete from pending where discord_id=$1 returning *",
        id
    )
    .fetch_one(pool)
    .await?;
    sqlx::query!(
        "insert into members values ($1, $2, $3, $4, $5)",
        id,
        p.shortcode,
        nickname,
        p.realname,
        fresher
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Add member entry to members table from manual table
pub(crate) async fn insert_member_from_manual(pool: &sqlx::PgPool, id: i64) -> Result<(), Error> {
    let m = sqlx::query_as!(
        ManualMember,
        "delete from manual where discord_id=$1 returning *",
        id
    )
    .fetch_one(pool)
    .await?;
    sqlx::query!(
        "insert into members values ($1, $2, $3, $4, $5)",
        id,
        m.shortcode,
        m.nickname,
        m.realname,
        m.fresher
    )
    .execute(pool)
    .await?;
    Ok(())
}
