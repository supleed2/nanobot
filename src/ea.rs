#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Member {
    pub first_name: String,
    pub surname: String,
    pub login: String,
    pub order_no: usize,
}

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
