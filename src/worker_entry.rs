use crate::{app::handle_calendar_url, AppError};
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    match handle_calendar_url(&req.url()?).await {
        Ok(Some(calendar)) => calendar_response(calendar),
        Ok(None) => Response::error("not found", 404),
        Err(err) => error_response(err),
    }
}

fn calendar_response(calendar: crate::RenderedCalendar) -> worker::Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", calendar.content_type)?;
    headers.set("Cache-Control", "public, max-age=1800")?;
    Ok(Response::from_body(ResponseBody::Body(calendar.body.into_bytes()))?.with_headers(headers))
}

fn error_response(err: AppError) -> worker::Result<Response> {
    Response::error(err.to_string(), err.status_code())
}
