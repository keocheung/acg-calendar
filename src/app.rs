use crate::{
    parse_calendar_route, AppResult, BangumiClient, BangumiGameWishlistSource, CalendarDataSource,
    CalendarKind, CalendarQuery, CalendarRequest, CalendarService, IcsRenderer, OutputFormat,
    RenderedCalendar,
};
use url::Url;

#[derive(Debug, Clone)]
pub struct CalendarApp<B> {
    bangumi_client: B,
}

impl<B> CalendarApp<B> {
    pub fn new(bangumi_client: B) -> Self {
        Self { bangumi_client }
    }
}

impl<B> CalendarApp<B>
where
    B: BangumiClient + Sync,
{
    pub async fn handle_url(&self, url: &Url) -> AppResult<Option<RenderedCalendar>> {
        let Some(route) = parse_calendar_route(url.path()) else {
            return Ok(None);
        };
        let query = CalendarQuery::from_url(url)?;
        let request = route.into_request(query);
        self.dispatch(request).await.map(Some)
    }

    async fn dispatch(&self, request: CalendarRequest) -> AppResult<RenderedCalendar> {
        match (&request.kind, &request.source, &request.format) {
            (
                CalendarKind::GameRelease,
                CalendarDataSource::BangumiUserCollection,
                OutputFormat::Ics,
            ) => {
                let service = CalendarService::new(
                    BangumiGameWishlistSource::new(&self.bangumi_client),
                    IcsRenderer,
                );
                service.generate(&request).await
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn handle_calendar_url(url: &Url) -> AppResult<Option<RenderedCalendar>> {
    CalendarApp::new(crate::ReqwestBangumiClient::default())
        .handle_url(url)
        .await
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn handle_calendar_url(url: &Url) -> AppResult<Option<RenderedCalendar>> {
    CalendarApp::new(crate::WorkerBangumiClient)
        .handle_url(url)
        .await
}
