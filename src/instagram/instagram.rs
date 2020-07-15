use chrono;
use reqwest::Client;
use serde_json::{json, Value};
use soup::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const PROFILE_URL: &str = "https://www.mystalk.net/profile/";
const LOAD_MORE_URL: &str = "https://www.mystalk.net/ajax/load-more/";
const DETAIL_URL: &str = "https://www.mystalk.net/detail/";

pub struct Builder {
    user: String,
    http_proxy: Option<String>,
    save_path: Option<PathBuf>,
}

impl Builder {
    pub fn new(user: String) -> Self {
        Self {
            user,
            http_proxy: None,
            save_path: None,
        }
    }

    pub fn http_proxy(mut self, http_proxy: &str) -> Self {
        self.http_proxy = Some(String::from(http_proxy));
        self
    }

    pub fn save_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.save_path = Some(PathBuf::from(path.as_ref()));
        self
    }

    pub fn build(self) -> Downloader {
        Downloader {
            http_proxy: self.http_proxy.clone(),
            save_path: self
                .save_path
                .as_ref()
                .map_or(PathBuf::new(), |path| PathBuf::from(path)),
            client: Client::builder()
                .cookie_store(true)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            user: self.user.clone(),
            profile_url: self
                .http_proxy
                .as_ref()
                .map_or(format!("{}{}/", PROFILE_URL, self.user), |http_proxy| {
                    format!("{}/proxy/{}{}/", http_proxy, PROFILE_URL, self.user)
                }),
            load_more_url: self
                .http_proxy
                .as_ref()
                .map_or(String::from(LOAD_MORE_URL), |http_proxy| {
                    format!("{}/proxy/{}", http_proxy, LOAD_MORE_URL)
                }),
            detail_url: self
                .http_proxy
                .as_ref()
                .map_or(String::from(DETAIL_URL), |http_proxy| {
                    format!("{}/proxy/{}", http_proxy, DETAIL_URL)
                }),
        }
    }
}

pub struct Downloader {
    http_proxy: Option<String>,
    save_path: PathBuf,
    client: Client,
    user: String,
    profile_url: String,
    load_more_url: String,
    detail_url: String,
}

impl Downloader {
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.create_dir()?;
        let csrf_token = self.get_profile().await?;
        let mut d: Value = json!({
            "more_available": true,
            "next_max_id": ""
        });
        let mut items = Vec::new();
        while d["more_available"].as_bool().unwrap() {
            let data = self
                .load_more(&csrf_token, d["next_max_id"].as_str().unwrap())
                .await?;
            self.save_raw(raw_file_name("loadmore.json"), data.as_bytes())?;
            d = serde_json::from_str(&data).unwrap();
            items.append(d["items"].as_array_mut().unwrap());
        }
        for item in items.iter() {
            match item["media_type"].as_i64().unwrap() {
                1 => self.save_media_type1(item).await?,
                2 => self.save_media_type2(item).await?,
                8 => self.save_media_type8(item).await?,
                _ => panic!("Invalid media type"),
            }
        }
        Ok(())
    }
}

// private methods
impl Downloader {
    fn create_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.save_path.join("instagram/raw"))?;
        fs::create_dir_all(self.save_path.join("instagram/media"))
    }

    fn save_raw<P: AsRef<Path>>(&self, name: P, data: &[u8]) -> std::io::Result<()> {
        let mut f = File::create(self.save_path.join("instagram/raw").join(name))?;
        f.write_all(data)?;
        Ok(())
    }

    async fn save_media_type1(&self, item: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.save_media(&extract_media_url(item, self.http_proxy.as_ref()))
            .await
    }

    async fn save_media_type2(&self, item: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let res = self
            .client
            .get(&format!(
                "{}{}/",
                self.detail_url,
                item["pk"].as_str().unwrap()
            ))
            .send()
            .await?;
        assert_eq!(res.status(), 200, "detail");
        self.save_media(&extract_video_url(
            &Soup::new(&res.text().await?),
            self.http_proxy.as_ref(),
        ))
        .await
    }

    async fn save_media_type8(&self, item: &Value) -> Result<(), Box<dyn std::error::Error>> {
        for item in item["carousel_media"].as_array().unwrap() {
            self.save_media(&extract_media_url(item, self.http_proxy.as_ref()))
                .await?;
        }
        Ok(())
    }

    async fn save_media(&self, media_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.client.get(media_url).send().await?;
        assert_eq!(res.status(), 200, "save media");
        let mut f = File::create(
            self.save_path.join("instagram/media").join(
                Url::parse(media_url)?
                    .path_segments()
                    .unwrap()
                    .last()
                    .unwrap(),
            ),
        )?;
        f.write_all(res.bytes().await?.as_ref())?;
        Ok(())
    }

    async fn get_profile(&mut self) -> reqwest::Result<String> {
        let res = self.client.get(&self.profile_url).send().await?;
        assert_eq!(res.status(), 200, "get profile");
        Ok(extract_csrf_token(&Soup::new(&res.text().await.unwrap())))
    }

    async fn load_more(&self, csrf_token: &str, page: &str) -> reqwest::Result<String> {
        let res = self
            .client
            .post(&self.load_more_url)
            .header("X-CSRFToken", csrf_token)
            .json(&json!({
                "name": self.user,
                "page": page,
                "type": "profile"
            }))
            .send()
            .await?;
        assert_eq!(res.status(), 200, "load more");
        res.text().await
    }
}

fn extract_media_url(item: &Value, http_proxy: Option<&String>) -> String {
    let url = item["image_versions2"].as_object().unwrap()["candidates"]
        .as_array()
        .unwrap()[2]
        .as_object()
        .unwrap()["url"]
        .as_str()
        .unwrap();
    format!("{}{}", http_proxy.map_or("", |http_proxy| http_proxy), &url)
}

fn extract_video_url(soup: &Soup, http_proxy: Option<&String>) -> String {
    let src = soup
        .tag("video")
        .find()
        .expect("Couldn't find tag video")
        .tag("source")
        .find()
        .expect("Couldn't find tag source")
        .get("src")
        .expect("Couldn't get src");
    format!("{}{}", http_proxy.map_or("", |http_proxy| http_proxy), &src)
}

fn extract_csrf_token(soup: &Soup) -> String {
    soup.tag("input")
        .find()
        .expect("Couldn't find tag input")
        .get("value")
        .expect("Couldn't get value")
}

fn raw_file_name(name: &str) -> String {
    format!(
        "{}_{}",
        chrono::offset::Local::now().format("%Y%m%d%H%M%S%6f"),
        name
    )
}
