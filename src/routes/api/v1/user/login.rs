use crate::app_error::AppError;
use crate::logic::{CancelUserDeletion, Login};
use crate::routes::utils::see_other_303;
use crate::session::UserSession;
use actix_web::http::header::LOCATION;
use actix_web::{post, web, HttpResponse};
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: Secret<String>,
    /// set to true if the user wish to cancel his account deletion.
    pub cancel_deletion: bool,
}

#[tracing::instrument(skip_all)]
#[post("/login")]
pub async fn login_user(
    web::Form(form): web::Form<LoginForm>,
    pg_pool: web::Data<PgPool>,
    session: UserSession,
) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        return Ok(see_other_303("/home"));
    }

    let creds = Login::validate_form_fields(form)?;
    let (user_id, requested_deletion) = creds.check_password_is_valid(&pg_pool).await?;
    if requested_deletion {
        if !creds.cancel_deletion {
            return Ok(HttpResponse::Conflict().finish());
        } else {
            CancelUserDeletion::remove_deletion_fields_with_user_id(&pg_pool, user_id).await?;
        }
    }
    session.activate(user_id)?;

    Ok(HttpResponse::Ok()
        .insert_header((LOCATION, "/home"))
        .finish())
}
