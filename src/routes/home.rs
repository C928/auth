use crate::app_error::AppError;
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse};

#[tracing::instrument(skip_all)]
#[get("/home")]
pub async fn get_home_page(session: UserSession) -> Result<HttpResponse, AppError> {
    if !session.is_active()? {
        return Ok(see_other_303("/login"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/home.html")))
}
