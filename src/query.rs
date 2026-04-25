use crate::{AppError, AppResult};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameMode {
    Original,
    Chinese,
}

#[derive(Debug, Clone)]
pub struct CalendarQuery {
    pub name: NameMode,
    pub past_days: i64,
}

impl Default for CalendarQuery {
    fn default() -> Self {
        Self {
            name: NameMode::Original,
            past_days: 31,
        }
    }
}

impl CalendarQuery {
    pub fn from_url(url: &Url) -> AppResult<Self> {
        let mut query = CalendarQuery::default();

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "name" | "title" => {
                    query.name = match value.as_ref() {
                        "original" | "orig" | "jp" | "ja" => NameMode::Original,
                        "cn" | "zh" | "chinese" => NameMode::Chinese,
                        other => {
                            return Err(AppError::BadRequest(format!(
                                "unsupported name mode: {other}"
                            )))
                        }
                    };
                }
                "past_days" => query.past_days = parse_non_negative_i64("past_days", &value)?,
                "past_months" => {
                    query.past_days = parse_non_negative_i64("past_months", &value)? * 31
                }
                "past" => query.past_days = parse_past(&value)?,
                _ => {}
            }
        }

        Ok(query)
    }
}

fn parse_non_negative_i64(name: &str, value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest(format!("{name} must be an integer")))?;
    if parsed < 0 {
        return Err(AppError::BadRequest(format!("{name} must be non-negative")));
    }
    Ok(parsed)
}

fn parse_past(value: &str) -> AppResult<i64> {
    if let Some(days) = value.strip_suffix('d') {
        parse_non_negative_i64("past", days)
    } else if let Some(months) = value.strip_suffix('m') {
        Ok(parse_non_negative_i64("past", months)? * 31)
    } else {
        parse_non_negative_i64("past", value)
    }
}
