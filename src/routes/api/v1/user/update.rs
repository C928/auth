use crate::app_error::AppError;
use crate::logic::{UpdateUser, UpdateUserError};
use crate::session::UserSession;
use actix_web::{post, web, HttpResponse};
use secrecy::Secret;
use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::PgPool;

#[serde_as]
#[derive(Deserialize)]
pub struct UpdateUserForm {
    #[serde_as(as = "NoneAsEmptyString")]
    pub new_email: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub new_username: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub new_password: Option<Secret<String>>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub new_password_confirm: Option<Secret<String>>,
    pub password: Secret<String>,
    pub confirmation_sentence: String,
}

#[post("/update")]
pub async fn update_user(
    web::Form(form): web::Form<UpdateUserForm>,
    pg_pool: web::Data<PgPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    let id = session.get_session_id()?;
    if form.confirmation_sentence != "Update my account." {
        Err(UpdateUserError::InvalidConfirmationSentence)?;
    }

    let creds = UpdateUser::validate_update_form(form)?;
    creds.check_password_is_valid(&pg_pool, &id).await?;
    creds.update_user_in_db(&pg_pool, id).await?;

    Ok(HttpResponse::NoContent().finish())
}
