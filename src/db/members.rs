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
    Ok(sqlx::query_as!(
        Member,
        "select * from members where lower(nickname)=lower($1)",
        nickname
    )
    .fetch_optional(pool)
    .await?)
}

/// Get member entry by Nickname (Fuzzy)
pub(crate) async fn get_member_by_nickname_fuzzy(
    pool: &sqlx::PgPool,
    nickname: &str,
    limit: i64,
) -> Result<Vec<Member>, Error> {
    Ok(sqlx::query_as!(
        Member,
        "select * from members where similarity(nickname,$1) > 0.3 order by similarity(nickname,$1) desc limit $2",
        nickname,
        limit,
    )
    .fetch_all(pool)
    .await?)
}

/// Get member entry by Real Name
pub(crate) async fn get_member_by_realname(
    pool: &sqlx::PgPool,
    realname: &str,
) -> Result<Option<Member>, Error> {
    Ok(sqlx::query_as!(
        Member,
        "select * from members where lower(realname)=lower($1)",
        realname
    )
    .fetch_optional(pool)
    .await?)
}

/// Get member entry by Real Name (Fuzzy)
pub(crate) async fn get_member_by_realname_fuzzy(
    pool: &sqlx::PgPool,
    realname: &str,
    limit: i64,
) -> Result<Vec<Member>, Error> {
    Ok(sqlx::query_as!(
        Member,
        "select * from members where similarity(realname,$1) > 0.3 order by similarity(realname,$1) desc limit $2",
        realname,
        limit,
    )
    .fetch_all(pool)
    .await?)
}

/// Add member entry to members table
pub(crate) async fn insert_member(pool: &sqlx::PgPool, m: Member) -> Result<(), Error> {
    sqlx::query!(
        "insert into members values ($1, $2, $3, $4, $5)",
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

/// Add member entry to members table from pending table
pub(crate) async fn insert_member_from_pending(
    pool: &sqlx::PgPool,
    id: i64,
    nickname: &str,
    fresher: bool,
) -> Result<Member, Error> {
    let p = sqlx::query_as!(
        PendingMember,
        "delete from pending where discord_id=$1 returning *",
        id
    )
    .fetch_one(pool)
    .await?;
    let m = sqlx::query_as!(
        Member,
        "insert into members values ($1, $2, $3, $4, $5) returning *",
        id,
        p.shortcode,
        nickname,
        p.realname,
        fresher
    )
    .fetch_one(pool)
    .await?;
    Ok(m)
}

/// Add member entry to members table from manual table
pub(crate) async fn insert_member_from_manual(
    pool: &sqlx::PgPool,
    id: i64,
) -> Result<Member, Error> {
    let mm = sqlx::query_as!(
        ManualMember,
        "delete from manual where discord_id=$1 returning *",
        id
    )
    .fetch_one(pool)
    .await?;
    let m = sqlx::query_as!(
        Member,
        "insert into members values ($1, $2, $3, $4, $5) returning *",
        id,
        mm.shortcode,
        mm.nickname,
        mm.realname,
        mm.fresher
    )
    .fetch_one(pool)
    .await?;
    Ok(m)
}

/// Edit member shortcode field
pub(crate) async fn edit_member_shortcode(
    pool: &sqlx::PgPool,
    id: i64,
    shortcode: &str,
) -> Result<bool, Error> {
    let r = sqlx::query!(
        "update members set shortcode=$2 where discord_id=$1",
        id,
        shortcode
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(r == 1)
}

/// Edit member nickname field
pub(crate) async fn edit_member_nickname(
    pool: &sqlx::PgPool,
    id: i64,
    nickname: &str,
) -> Result<bool, Error> {
    let r = sqlx::query!(
        "update members set nickname=$2 where discord_id=$1",
        id,
        nickname
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(r == 1)
}

/// Edit member realname field
pub(crate) async fn edit_member_realname(
    pool: &sqlx::PgPool,
    id: i64,
    realname: &str,
) -> Result<bool, Error> {
    let r = sqlx::query!(
        "update members set realname=$2 where discord_id=$1",
        id,
        realname
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(r == 1)
}

/// Edit member fresher field
pub(crate) async fn edit_member_fresher(
    pool: &sqlx::PgPool,
    id: i64,
    fresher: bool,
) -> Result<bool, Error> {
    let r = sqlx::query!(
        "update members set fresher=$2 where discord_id=$1",
        id,
        fresher
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(r == 1)
}

/// Set all members to non-freshers
pub(crate) async fn set_members_non_fresher(pool: &sqlx::PgPool) -> Result<u64, Error> {
    Ok(sqlx::query!("update members set fresher='f'")
        .execute(pool)
        .await?
        .rows_affected())
}
