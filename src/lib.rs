mod app;
mod bangumi;
mod domain;
mod error;
mod logging;
mod query;
mod renderers;
mod route;
mod service;

#[cfg(target_arch = "wasm32")]
mod worker_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use bangumi::ReqwestBangumiClient;
#[cfg(target_arch = "wasm32")]
pub(crate) use bangumi::WorkerBangumiClient;
pub use bangumi::{
    BangumiClient, BangumiCollectionItem, BangumiCollectionPage, BangumiGameWishlistSource,
    BangumiSubject,
};
pub use domain::{
    CalendarDataSource, CalendarDocument, CalendarEvent, CalendarKind, CalendarRequest,
    OutputFormat, RenderedCalendar,
};
pub use error::{AppError, AppResult};
pub use query::{CalendarQuery, NameMode};
pub use renderers::IcsRenderer;
pub use route::{parse_calendar_route, CalendarRoute};
pub use service::{CalendarRenderer, CalendarService, CalendarSource};

pub async fn build_bangumi_game_calendar<C: BangumiClient + Sync>(
    client: &C,
    username: &str,
    query: &CalendarQuery,
) -> AppResult<String> {
    let request = CalendarRequest {
        kind: CalendarKind::GameRelease,
        source: CalendarDataSource::BangumiUserCollection,
        format: OutputFormat::Ics,
        owner: username.to_owned(),
        query: query.clone(),
    };
    let service = CalendarService::new(BangumiGameWishlistSource::new(client), IcsRenderer);
    Ok(service.generate(&request).await?.body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use calcard::{Entry, Parser};
    use chrono::{Duration, NaiveDate, Utc};
    use url::Url;

    struct FakeBangumiClient {
        pages: Vec<BangumiCollectionPage>,
    }

    impl BangumiClient for FakeBangumiClient {
        async fn user_game_wishlist_page(
            &self,
            _username: &str,
            _limit: u16,
            offset: u32,
        ) -> AppResult<BangumiCollectionPage> {
            let index = (offset / bangumi::PAGE_LIMIT as u32) as usize;
            Ok(self
                .pages
                .get(index)
                .cloned()
                .unwrap_or(BangumiCollectionPage {
                    total: 0,
                    data: Vec::new(),
                }))
        }
    }

    #[test]
    fn parses_calendar_route() {
        let route = parse_calendar_route("/game/bangumi/user/tom/wish.ics").unwrap();
        assert_eq!(route.username, "tom");
        assert!(parse_calendar_route("/game/bangumi/user/tom").is_none());
    }

    #[test]
    fn parses_query_defaults_and_overrides() {
        let url = Url::parse("https://example.com/game/bangumi/user/tom/wish.ics").unwrap();
        let query = CalendarQuery::from_url(&url).unwrap();
        assert_eq!(query.name, NameMode::Original);
        assert_eq!(query.past_days, 31);

        let url = Url::parse("https://example.com/game/bangumi/user/tom/wish.ics?name=cn&past=2m")
            .unwrap();
        let query = CalendarQuery::from_url(&url).unwrap();
        assert_eq!(query.name, NameMode::Chinese);
        assert_eq!(query.past_days, 62);
    }

    #[tokio::test]
    async fn generates_parseable_ics_with_filtered_games() {
        let today = Utc::now().date_naive();
        let included = today + Duration::days(10);
        let old = today - Duration::days(90);
        let client = FakeBangumiClient {
            pages: vec![BangumiCollectionPage {
                total: 2,
                data: vec![
                    item(1, "Original Game", "中文游戏", included, "PC"),
                    item(2, "Old Game", "旧游戏", old, "Switch"),
                ],
            }],
        };

        let ics = build_bangumi_game_calendar(
            &client,
            "tom",
            &CalendarQuery {
                name: NameMode::Chinese,
                past_days: 31,
            },
        )
        .await
        .unwrap();

        assert!(ics.contains("BEGIN:VCALENDAR"));
        assert!(ics.contains("BEGIN:VEVENT"));
        assert!(ics.contains("SUMMARY:中文游戏"));
        assert!(!ics.contains("Old Game"));

        let mut parser = Parser::new(&ics);
        match parser.entry() {
            Entry::ICalendar(_) => {}
            other => panic!("expected iCalendar, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn service_composes_source_and_renderer() {
        let today = Utc::now().date_naive();
        let client = FakeBangumiClient {
            pages: vec![BangumiCollectionPage {
                total: 1,
                data: vec![item(10, "Service Game", "", today, "PC")],
            }],
        };
        let request = CalendarRequest {
            kind: CalendarKind::GameRelease,
            source: CalendarDataSource::BangumiUserCollection,
            format: OutputFormat::Ics,
            owner: "tom".to_owned(),
            query: CalendarQuery::default(),
        };
        let service = CalendarService::new(BangumiGameWishlistSource::new(client), IcsRenderer);

        let rendered = service.generate(&request).await.unwrap();

        assert_eq!(rendered.content_type, "text/calendar; charset=utf-8");
        assert!(rendered.body.contains("SUMMARY:Service Game"));
        assert!(rendered.body.contains("NAME:Bangumi tom Game Releases"));
        assert!(rendered
            .body
            .contains("X-WR-CALNAME:Bangumi tom Game Releases"));
    }

    fn item(
        id: u64,
        name: &str,
        name_cn: &str,
        date: NaiveDate,
        platform: &str,
    ) -> BangumiCollectionItem {
        BangumiCollectionItem {
            subject: BangumiSubject {
                id,
                name: name.to_owned(),
                name_cn: name_cn.to_owned(),
                summary: "line one\nline two".to_owned(),
                date: date.format("%Y-%m-%d").to_string(),
                platform: platform.to_owned(),
                url: format!("https://bgm.tv/subject/{id}"),
            },
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use app::handle_calendar_url;
pub use app::CalendarApp;
