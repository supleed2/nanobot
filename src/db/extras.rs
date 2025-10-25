use crate::{Error, Gaijin};

/// Get count of entries in gaijin table
pub(crate) async fn count_gaijin(pool: &sqlx::SqlitePool) -> Result<i64, Error> {
    Ok(sqlx::query!("select count(*) as \"i64!\" from gaijin")
        .fetch_one(pool)
        .await?
        .i64)
}

/// Delete gaijin by Discord ID
pub(crate) async fn delete_gaijin_by_id(pool: &sqlx::SqlitePool, id: i64) -> Result<bool, Error> {
    let r = sqlx::query!("delete from gaijin where discord_id=$1", id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(r == 1)
}

/// Get all entries in gaijin table
pub(crate) async fn get_all_gaijin(pool: &sqlx::SqlitePool) -> Result<Vec<Gaijin>, Error> {
    Ok(sqlx::query_as!(Gaijin, "select * from gaijin")
        .fetch_all(pool)
        .await?)
}

/// Get gaijin entry by Discord ID
pub(crate) async fn get_gaijin_by_id(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<Option<Gaijin>, Error> {
    Ok(
        sqlx::query_as!(Gaijin, "select * from gaijin where discord_id=$1", id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Get gaijin entry by Name
pub(crate) async fn get_gaijin_by_name(
    pool: &sqlx::SqlitePool,
    name: &str,
) -> Result<Option<Gaijin>, Error> {
    Ok(sqlx::query_as!(
        Gaijin,
        "select * from gaijin where lower(name)=lower($1)",
        name
    )
    .fetch_optional(pool)
    .await?)
}

// /// Get gaijin entry by Name (Fuzzy) TODO: add to whois
// pub(crate) async fn get_gaijin_by_name_fuzzy(
//     pool: &sqlx::SqlitePool,
//     name: &str,
//     limit: i64,
// ) -> Result<Vec<Gaijin>, Error> {
//     Ok(sqlx::query_as!(
//         Gaijin,
//         "select * from gaijin where similarity(name,$1) > 0.3 order by similarity(name,$1) desc limit $2",
//         name,
//         limit
//     )
//     .fetch_all(pool)
//     .await?)
// }

/// Add entry to gaijin table
pub(crate) async fn insert_gaijin(pool: &sqlx::SqlitePool, g: Gaijin) -> Result<(), Error> {
    sqlx::query!(
        "insert into gaijin values ($1, $2, $3)",
        g.discord_id,
        g.name,
        g.university
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Edit gaijin name field
pub(crate) async fn edit_gaijin_name(
    pool: &sqlx::SqlitePool,
    id: i64,
    name: &str,
) -> Result<bool, Error> {
    let r = sqlx::query!("update gaijin set name=$2 where discord_id=$1", id, name)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(r == 1)
}

/// Edit gaijin university field
pub(crate) async fn edit_gaijin_university(
    pool: &sqlx::SqlitePool,
    id: i64,
    university: &str,
) -> Result<bool, Error> {
    let r = sqlx::query!(
        "update gaijin set university=$2 where discord_id=$1",
        id,
        university
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(r == 1)
}
