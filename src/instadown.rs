use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

static INSTA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://(www\.)?(instagram\.com|instagr\.am)(/share)?/").unwrap()
});

const DD_INSTAGRAM: &str = "https://d.ddinstagram.com/";

pub async fn instadown(url: &str, client: &Client) -> Result<String, crate::Error> {
    let x = INSTA_REGEX.replace_all(url, DD_INSTAGRAM);
    let y = client.get(&*x).send().await?;
    let z = y.url().to_string();
    Ok(z)
}
