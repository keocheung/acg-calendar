use crate::{AppResult, CalendarDocument, CalendarRequest, RenderedCalendar};

pub trait CalendarSource {
    #[allow(async_fn_in_trait)]
    async fn load_calendar(&self, request: &CalendarRequest) -> AppResult<CalendarDocument>;
}

pub trait CalendarRenderer {
    fn render(&self, calendar: &CalendarDocument) -> AppResult<RenderedCalendar>;
}

pub struct CalendarService<S, R> {
    source: S,
    renderer: R,
}

impl<S, R> CalendarService<S, R> {
    pub fn new(source: S, renderer: R) -> Self {
        Self { source, renderer }
    }
}

impl<S, R> CalendarService<S, R>
where
    S: CalendarSource + Sync,
    R: CalendarRenderer + Sync,
{
    pub async fn generate(&self, request: &CalendarRequest) -> AppResult<RenderedCalendar> {
        let document = self.source.load_calendar(request).await?;
        self.renderer.render(&document)
    }
}
