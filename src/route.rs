use crate::{CalendarDataSource, CalendarKind, CalendarQuery, CalendarRequest, OutputFormat};

#[derive(Debug, Clone)]
pub struct CalendarRoute {
    pub username: String,
}

pub fn parse_calendar_route(path: &str) -> Option<CalendarRoute> {
    let parts = path.trim_matches('/').split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        ["game", "bangumi", "user", username, "wish.ics"] if !username.is_empty() => {
            Some(CalendarRoute {
                username: (*username).to_owned(),
            })
        }
        _ => None,
    }
}

impl CalendarRoute {
    pub fn into_request(self, query: CalendarQuery) -> CalendarRequest {
        CalendarRequest {
            kind: CalendarKind::GameRelease,
            source: CalendarDataSource::BangumiUserCollection,
            format: OutputFormat::Ics,
            owner: self.username,
            query,
        }
    }
}
