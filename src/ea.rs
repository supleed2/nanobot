#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EaMember {
    pub first_name: String,
    pub surname: String,
    pub login: String,
    pub order_no: usize,
}

pub(crate) async fn get_members_list(
    api_key: &str,
    url: &str,
) -> Result<Vec<EaMember>, reqwest::Error> {
    let members = reqwest::Client::new()
        .get(url)
        .header("X-API-Key", api_key)
        .send()
        .await?
        .json::<Vec<EaMember>>()
        .await?;
    Ok(members)
}
