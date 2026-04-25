use std::fmt;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Upstream(String),
    Calendar(String),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Upstream(_) => 502,
            Self::Calendar(_) => 500,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(message) | Self::Upstream(message) | Self::Calendar(message) => {
                f.write_str(message)
            }
        }
    }
}

impl std::error::Error for AppError {}
