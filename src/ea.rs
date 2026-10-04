#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Member {
    pub first_name: String,
    pub surname: String,
    #[serde(rename = "CID")]
    pub cid: String,
    pub login: String,
    pub order_no: usize,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_members_list(
    api_key: &str,
    url: &str,
) -> Result<Vec<Member>, reqwest::Error> {
    let members = reqwest::Client::new()
        .get(url)
        .header("X-API-Key", api_key)
        .send()
        .await?
        .json::<Vec<Member>>()
        .await?;
    Ok(members)
}

pub(crate) fn current_url(url: &str) -> String {
    use chrono::Datelike as _;

    let now = chrono::Utc::now();
    let year = now.year() - i32::from(now.month() < 10);
    let (start, end) = (year % 100, (year + 1) % 100);

    format!("{url}{start:02}-{end:02}")
}
