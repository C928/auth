use crate::app_error::AppError;
use crate::logic::FieldValidationError;

pub fn sqlx_user_insertion_error(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(ref dbe) = err {
        match dbe.constraint() {
            Some("users_email_key") => FieldValidationError::EmailTaken.into(),
            Some("users_username_key") => FieldValidationError::UsernameTaken.into(),
            _ => AppError::with_msg(err.to_string()),
        }
    } else {
        err.into()
    }
}
