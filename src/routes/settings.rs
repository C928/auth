use crate::app_error::AppError;
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse};

#[get("/settings")]
pub async fn get_settings_page(session: UserSession) -> Result<HttpResponse, AppError> {
    if !session.is_active()? {
        return Ok(see_other_303("/login"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/settings.html")))
}
