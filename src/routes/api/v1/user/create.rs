use crate::app_error::AppError;
use crate::db::get_redis_connection;
use crate::logic::{CaptchaAnswer, CreateUser, CreateUserRequest, URLToken};
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::LOCATION;
use actix_web::{post, web, HttpResponse};
use deadpool_redis::Pool as RedisPool;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CreateUserRequestForm {
    pub email: String,
    pub captcha_id: String,
    pub captcha_answer: String,
    pub bzz: Option<String>,
}

#[tracing::instrument(skip_all)]
#[post("/create/request")]
pub async fn create_user_request(
    web::Form(form): web::Form<CreateUserRequestForm>,
    pg_pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    }

    let creds = CreateUserRequest::validate_register_request_form(form)?;
    creds.check_email_taken(&pg_pool).await?;

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
pub struct CreateUserForm {
    pub token: String,
    pub username: String,
    pub password: Secret<String>,
    pub password_confirm: Secret<String>,
}

#[tracing::instrument(skip_all)]
#[post("/create")]
pub async fn create_user(
    web::Form(form): web::Form<CreateUserForm>,
    pg_pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    let creds = CreateUser::validate_register_form(form)?;
    let redis_conn = get_redis_connection(&redis_pool).await?;
    creds.check_username_taken(&pg_pool).await?;

    let user_fields = creds.token.get_associated_redis_fields(redis_conn).await?;
    let user_id = creds
        .insert_user_infos_to_db(&pg_pool, user_fields.email)
        .await?;
    session.activate(user_id)?;

    Ok(HttpResponse::Created()
        .insert_header((LOCATION, "/home"))
        .finish())
}
