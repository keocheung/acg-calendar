use crate::CalendarQuery;
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarKind {
    GameRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarDataSource {
    BangumiUserCollection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Ics,
}

#[derive(Debug, Clone)]
pub struct CalendarRequest {
    pub kind: CalendarKind,
    pub source: CalendarDataSource,
    pub format: OutputFormat,
    pub owner: String,
    pub query: CalendarQuery,
}

#[derive(Debug, Clone)]
pub struct CalendarDocument {
    pub name: String,
    pub events: Vec<CalendarEvent>,
}

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub uid: String,
    pub title: String,
    pub date: NaiveDate,
    pub url: Option<String>,
    pub description: Option<String>,
}

pub struct RenderedCalendar {
    pub body: String,
    pub content_type: &'static str,
}
