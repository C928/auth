use crate::app_error::AppError;
use crate::logic::{Email, FieldValidationError, Password, PasswordHash};
use crate::routes::LoginForm;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthError {
    InvalidCredentials,
    InvalidPassword,
}

pub struct Login {
    pub email: Email,
    pub password: Password,
    pub cancel_deletion: bool,
}

impl Login {
    pub fn validate_form_fields(form: LoginForm) -> Result<Self, FieldValidationError> {
        Ok(Self {
            email: Email::parse(form.email)?,
            password: Password::parse(form.password)?,
            cancel_deletion: form.cancel_deletion,
        })
    }

    pub async fn check_password_is_valid(&self, pool: &PgPool) -> Result<(Uuid, bool), AppError> {
        let ret = sqlx::query!(
            "select id, password_hash, requested_deletion from users where email = $1",
            self.email.as_str(),
        )
        .fetch_optional(pool)
        .await?;

        if let Some(infos) = ret {
            let hash = PasswordHash::from_str(infos.password_hash);
            self.password.verify_password(&hash)?;
            Ok((
                infos.id,
                infos.requested_deletion.map(|_| true).unwrap_or(false),
            ))
        } else {
            Err(AuthError::InvalidCredentials)?
        }
    }
}
