use crate::app_error::AppError;
use crate::db::get_redis_connection;
use crate::logic::{CaptchaAnswer, ResetPassword, ResetPasswordRequest, URLToken};
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::LOCATION;
use actix_web::{post, web, HttpResponse};
use deadpool_redis::Pool as RedisPool;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct ResetPasswordRequestForm {
    pub email: String,
    pub captcha_id: String,
    pub captcha_answer: String,
    pub bzz: Option<String>,
}

#[tracing::instrument(skip_all)]
#[post("/reset-password/request")]
pub async fn reset_user_password_request(
    web::Form(form): web::Form<ResetPasswordRequestForm>,
    redis_pool: web::Data<RedisPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    }

    let creds = ResetPasswordRequest::validate_password_reset_request_form(form)?;
    let mut redis_conn = get_redis_connection(&redis_pool).await?;
    CaptchaAnswer::is_valid_captcha_answer(
        &mut redis_conn,
        &creds.captcha_answer,
        &creds.captcha_id,
    )
    .await?;

    let url_token = URLToken::store_user_fields_to_redis(redis_conn, &creds.email).await?;
    creds.send_confirmation_email(url_token).await;

    Ok(HttpResponse::Accepted().finish())
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub new_password: Secret<String>,
    pub new_password_confirm: Secret<String>,
}

#[tracing::instrument(skip_all)]
#[post("/reset-password")]
pub async fn reset_user_password(
    web::Form(form): web::Form<ResetPasswordForm>,
    pg_pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, AppError> {
    let creds = ResetPassword::validate_reset_password_form(form)?;
    let redis_conn = get_redis_connection(&redis_pool).await?;

    let user_fields = creds.token.get_associated_redis_fields(redis_conn).await?;
    let update_fields = creds
        .update_password_in_db(&pg_pool, user_fields.email)
        .await?;
    update_fields.send_password_updated_email().await;

    Ok(HttpResponse::NoContent()
        .insert_header((LOCATION, "/login"))
        .finish())
}
