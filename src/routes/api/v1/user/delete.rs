use crate::app_error::AppError;
use crate::logic::{CancelUserDeletion, DeleteUserRequest, UpdateUserError};
use crate::session::UserSession;
use actix_web::http::header::LOCATION;
use actix_web::{get, post, web, HttpResponse};
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct DeleteUserRequestForm {
    pub password: Secret<String>,
    pub confirmation_sentence: String,
}

#[tracing::instrument(skip_all)]
#[post("/delete/request")]
pub async fn delete_user_request(
    web::Form(form): web::Form<DeleteUserRequestForm>,
    pg_pool: web::Data<PgPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    let id = session.get_session_id()?;
    if form.confirmation_sentence != "Delete my account." {
        Err(UpdateUserError::InvalidConfirmationSentence)?;
    }

    let creds = DeleteUserRequest::validate_delete_user_request_form(form, id)?;
    creds.verify_password(&pg_pool).await?;

    let (cancel_token, user_infos) = creds.insert_account_deletion_entry_to_db(&pg_pool).await?;
    session.deactivate();
    DeleteUserRequest::send_account_deletion_requested_email(cancel_token, user_infos).await;

    Ok(HttpResponse::Accepted()
        .insert_header((LOCATION, "/login"))
        .finish())
}

#[derive(Deserialize)]
pub struct Token {
    pub token: String,
}

#[tracing::instrument(skip_all)]
#[get("/delete/cancel")]
pub async fn cancel_delete_user_request(
    web::Query(param): web::Query<Token>,
    pg_pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let token = CancelUserDeletion::from_url_token(param.token)?;
    token.remove_deletion_fields_with_token(&pg_pool).await?;

    Ok(HttpResponse::Ok()
        .insert_header((LOCATION, "/login"))
        .finish())
}
