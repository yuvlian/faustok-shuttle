use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static API_BASE_URL: LazyLock<String> =
    LazyLock::new(|| String::from("https://api.tiklydown.eu.org/api/download?url="));

#[derive(Serialize, Deserialize)]
pub struct TiklydownRsp {
    pub images: Option<Vec<TiklydownImg>>,
    pub video: Option<TiklydownVid>,
    pub music: Option<TiklydownMsc>,
}

impl TiklydownRsp {
    pub async fn fetch_url(url: &str, client: &Client) -> Result<Self, crate::Error> {
        let x = format!("{}{}", *API_BASE_URL, url);
        let y = client.get(&x).send().await?;
        let z: Self = y.json().await?;
        Ok(z)
    }

    pub fn get_media_urls(&self, get_music: bool) -> (Vec<&str>, Option<&str>) {
        let mut media_urls = Vec::with_capacity(1);

        if let Some(images) = &self.images {
            media_urls.extend(images.iter().map(|img| img.url.as_str()));
        }

        if let Some(video) = &self.video {
            media_urls.push(video.url.as_str());
        }

        let music_url = if get_music {
            self.music.as_ref().map(|msc| msc.url.as_str())
        } else {
            None
        };

        (media_urls, music_url)
    }
}

#[derive(Serialize, Deserialize)]
pub struct TiklydownImg {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct TiklydownVid {
    #[serde(rename = "noWatermark")]
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct TiklydownMsc {
    #[serde(rename = "play_url")]
    pub url: String,
}
