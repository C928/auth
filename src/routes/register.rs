use crate::app_error::AppError;
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

#[get("/register/request")]
pub async fn get_register_request_page(session: UserSession) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/register-request.html")))
}

#[derive(Deserialize)]
pub struct RegisterToken {
    token: Option<String>,
}

#[get("/register")]
pub async fn get_register_page(
    web::Query(param): web::Query<RegisterToken>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    } else if param.token.is_none() {
        return Ok(see_other_303("/register/request"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/register.html")))
}
