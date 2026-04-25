use crate::{
    logging, AppError, AppResult, CalendarDataSource, CalendarDocument, CalendarEvent,
    CalendarKind, CalendarRequest, CalendarSource, NameMode,
};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Deserializer};

const BANGUMI_API: &str = "https://api.bgm.tv";
const GAME_SUBJECT_TYPE: u8 = 4;
const WISH_COLLECTION_TYPE: u8 = 1;
pub(crate) const PAGE_LIMIT: u16 = 100;
const MAX_PAGES: u16 = 50;

#[derive(Debug, Clone, Deserialize)]
pub struct BangumiCollectionPage {
    pub total: u32,
    pub data: Vec<BangumiCollectionItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BangumiCollectionItem {
    pub subject: BangumiSubject,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BangumiSubject {
    pub id: u64,
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub name_cn: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub summary: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub date: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub platform: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub url: String,
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

impl BangumiSubject {
    fn release_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }

    fn display_name(&self, mode: &NameMode) -> &str {
        match mode {
            NameMode::Original => &self.name,
            NameMode::Chinese if !self.name_cn.trim().is_empty() => &self.name_cn,
            NameMode::Chinese => &self.name,
        }
    }

    fn subject_url(&self) -> String {
        if self.url.trim().is_empty() {
            format!("https://bgm.tv/subject/{}", self.id)
        } else {
            self.url.clone()
        }
    }

    fn into_event(self, release_date: NaiveDate, name_mode: &NameMode) -> CalendarEvent {
        let title = self.display_name(name_mode).to_owned();
        let url = self.subject_url();
        CalendarEvent {
            uid: format!("bangumi-game-{}@calendar.local", self.id),
            title,
            date: release_date,
            description: Some(event_description(&self)),
            url: Some(url),
        }
    }
}

pub trait BangumiClient {
    #[allow(async_fn_in_trait)]
    async fn user_game_wishlist_page(
        &self,
        username: &str,
        limit: u16,
        offset: u32,
    ) -> AppResult<BangumiCollectionPage>;
}

impl<T> BangumiClient for &T
where
    T: BangumiClient + Sync + ?Sized,
{
    async fn user_game_wishlist_page(
        &self,
        username: &str,
        limit: u16,
        offset: u32,
    ) -> AppResult<BangumiCollectionPage> {
        (**self)
            .user_game_wishlist_page(username, limit, offset)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct BangumiGameWishlistSource<C> {
    client: C,
}

impl<C> BangumiGameWishlistSource<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> CalendarSource for BangumiGameWishlistSource<C>
where
    C: BangumiClient + Sync,
{
    async fn load_calendar(&self, request: &CalendarRequest) -> AppResult<CalendarDocument> {
        if request.kind != CalendarKind::GameRelease
            || request.source != CalendarDataSource::BangumiUserCollection
        {
            return Err(AppError::BadRequest(
                "unsupported calendar source request".to_owned(),
            ));
        }

        let today = Utc::now().date_naive();
        let since = today - Duration::days(request.query.past_days);
        let mut events = Vec::new();
        let mut offset = 0;

        for _ in 0..MAX_PAGES {
            let page = self
                .client
                .user_game_wishlist_page(&request.owner, PAGE_LIMIT, offset)
                .await?;
            let total = page.total;

            events.extend(page.data.into_iter().filter_map(|item| {
                let release_date = item.subject.release_date()?;
                (release_date >= since)
                    .then(|| item.subject.into_event(release_date, &request.query.name))
            }));

            offset += u32::from(PAGE_LIMIT);
            if offset >= total {
                break;
            }
        }

        events.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.uid.cmp(&b.uid)));

        Ok(CalendarDocument {
            name: format!("Bangumi {} Game Releases", request.owner),
            events,
        })
    }
}

pub(crate) fn bangumi_collections_url(username: &str, limit: u16, offset: u32) -> String {
    format!(
        "{BANGUMI_API}/v0/users/{username}/collections?subject_type={GAME_SUBJECT_TYPE}&type={WISH_COLLECTION_TYPE}&limit={limit}&offset={offset}"
    )
}

pub(crate) fn parse_collection_page(
    url: &str,
    status: u16,
    content_type: Option<&str>,
    body: &str,
) -> AppResult<BangumiCollectionPage> {
    serde_json::from_str::<BangumiCollectionPage>(body).map_err(|err| {
        logging::error(&format!(
            "Bangumi response parse failed url={url} status={status} content_type={} error={err} body_preview=\"{}\"",
            content_type.unwrap_or("<missing>"),
            body_preview(body)
        ));
        AppError::Upstream(format!("Bangumi response parse failed: {err}"))
    })
}

fn event_description(subject: &BangumiSubject) -> String {
    let mut lines = vec![subject.subject_url()];
    if !subject.platform.trim().is_empty() {
        lines.push(format!("Platform: {}", subject.platform));
    }
    if !subject.summary.trim().is_empty() {
        lines.push(subject.summary.clone());
    }
    lines.join("\n")
}

pub(crate) fn body_preview(body: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 1200;
    let mut preview = body
        .chars()
        .take(MAX_PREVIEW_CHARS)
        .collect::<String>()
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    if body.chars().count() > MAX_PREVIEW_CHARS {
        preview.push_str("...");
    }
    preview
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ReqwestBangumiClient {
    http: reqwest::Client,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ReqwestBangumiClient {
    fn default() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("calendar/0.1 (https://github.com/keocheung/calendar)")
                .build()
                .expect("reqwest client should build"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BangumiClient for ReqwestBangumiClient {
    async fn user_game_wishlist_page(
        &self,
        username: &str,
        limit: u16,
        offset: u32,
    ) -> AppResult<BangumiCollectionPage> {
        let url = bangumi_collections_url(username, limit, offset);
        logging::info(&format!("requesting Bangumi collections url={url}"));
        let response = self.http.get(&url).send().await.map_err(|err| {
            logging::error(&format!("Bangumi request failed url={url} error={err}"));
            AppError::Upstream(format!("Bangumi request failed: {err}"))
        })?;
        let status = response.status();
        let status_code = status.as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response.text().await.map_err(|err| {
            logging::error(&format!(
                "Bangumi response body read failed url={url} status={status_code} error={err}"
            ));
            AppError::Upstream(format!("Bangumi response body read failed: {err}"))
        })?;
        logging::info(&format!(
            "Bangumi response received url={url} status={status_code} content_type={} bytes={}",
            content_type.as_deref().unwrap_or("<missing>"),
            body.len()
        ));
        if !status.is_success() {
            logging::error(&format!(
                "Bangumi returned non-success url={url} status={status_code} content_type={} body_preview=\"{}\"",
                content_type.as_deref().unwrap_or("<missing>"),
                body_preview(&body)
            ));
            return Err(AppError::Upstream(format!(
                "Bangumi returned HTTP {}",
                status_code
            )));
        }
        parse_collection_page(&url, status_code, content_type.as_deref(), &body)
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, Default)]
pub struct WorkerBangumiClient;

#[cfg(target_arch = "wasm32")]
impl BangumiClient for WorkerBangumiClient {
    async fn user_game_wishlist_page(
        &self,
        username: &str,
        limit: u16,
        offset: u32,
    ) -> AppResult<BangumiCollectionPage> {
        use worker::*;

        let url = bangumi_collections_url(username, limit, offset);
        logging::info(&format!("requesting Bangumi collections url={url}"));
        let headers = Headers::new();
        headers
            .set("Accept", "application/json")
            .map_err(|err| AppError::Upstream(format!("failed to set header: {err:?}")))?;
        headers
            .set("User-Agent", "calendar/0.1")
            .map_err(|err| AppError::Upstream(format!("failed to set header: {err:?}")))?;
        let request = Request::new_with_init(
            &url,
            &RequestInit::new()
                .with_method(Method::Get)
                .with_headers(headers),
        )
        .map_err(|err| AppError::Upstream(format!("failed to build request: {err:?}")))?;
        let mut response = Fetch::Request(request).send().await.map_err(|err| {
            logging::error(&format!("Bangumi request failed url={url} error={err:?}"));
            AppError::Upstream(format!("Bangumi request failed: {err:?}"))
        })?;
        let status_code = response.status_code();
        let content_type = response.headers().get("content-type").ok().flatten();
        let body = response.text().await.map_err(|err| {
            logging::error(&format!(
                "Bangumi response body read failed url={url} status={status_code} error={err:?}"
            ));
            AppError::Upstream(format!("Bangumi response body read failed: {err:?}"))
        })?;
        logging::info(&format!(
            "Bangumi response received url={url} status={status_code} content_type={} bytes={}",
            content_type.as_deref().unwrap_or("<missing>"),
            body.len()
        ));
        if status_code >= 400 {
            logging::error(&format!(
                "Bangumi returned non-success url={url} status={status_code} content_type={} body_preview=\"{}\"",
                content_type.as_deref().unwrap_or("<missing>"),
                body_preview(&body)
            ));
            return Err(AppError::Upstream(format!(
                "Bangumi returned HTTP {}",
                status_code
            )));
        }
        parse_collection_page(&url, status_code, content_type.as_deref(), &body)
    }
}
