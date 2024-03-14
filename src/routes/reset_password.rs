use crate::app_error::AppError;
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

#[get("/reset-password/request")]
pub async fn get_reset_password_request_page(
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!(
            "../../static/html/reset-password-request.html"
        )))
}

#[derive(Deserialize)]
pub struct ResetPasswordToken {
    token: Option<String>,
}

#[get("/reset-password")]
pub async fn get_reset_password_page(
    session: UserSession,
    web::Query(param): web::Query<ResetPasswordToken>,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    } else if param.token.is_none() {
        return Ok(see_other_303("/reset-password/request"));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/reset-password.html")))
}
