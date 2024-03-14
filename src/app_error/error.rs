use crate::logic::{AuthError, CreateUserError, FieldValidationError, UpdateUserError};
use crate::session::UserSessionError;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::anyhow;
use serde::Serialize;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorType {
    ValidationError(FieldValidationError),
    RegistrationError(CreateUserError),
    UpdateUserError(UpdateUserError),
    AuthError(AuthError),
    SessionError(UserSessionError),
    Unknown(()),
}

pub struct AppError {
    pub error_type: AppErrorType,
    pub msg: Option<anyhow::Error>,
    //todo: add trace to app_error ?
}

impl AppError {
    pub fn with_msg(msg: String) -> Self {
        Self {
            error_type: AppErrorType::Unknown(()),
            msg: Some(anyhow!(msg)),
        }
    }
}

impl Debug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppError")
            .field("error_type", &self.error_type)
            .field("msg", &self.msg)
            .finish()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "error_type: {:?}\nmsg: {:?}", self.error_type, self.msg)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(StatusCode::BAD_REQUEST).json(&self.error_type)
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error> + Debug,
{
    fn from(error: E) -> Self {
        Self {
            error_type: AppErrorType::Unknown(()),
            msg: Some(error.into()),
        }
    }
}

impl From<AppErrorType> for AppError {
    fn from(error_type: AppErrorType) -> Self {
        Self {
            error_type,
            msg: None,
        }
    }
}

/*
//todo: impl trait to these errors to make 1 conversion impl
//todo: use procedural macro to impl this trait to each sub app_error
trait ToAppError {}
impl ToAppError for FieldValidationError {}
impl<E> From<E> for AppError where E: ToAppError {
    fn from(app_error: E) -> Self {
        Self {
            error_type: AppErrorType::ValidationError(app_error),
            msg: None,
        }
    }
}
*/

impl From<FieldValidationError> for AppError {
    fn from(error: FieldValidationError) -> Self {
        Self {
            error_type: AppErrorType::ValidationError(error),
            msg: None,
        }
    }
}

impl From<CreateUserError> for AppError {
    fn from(error: CreateUserError) -> Self {
        Self {
            error_type: AppErrorType::RegistrationError(error),
            msg: None,
        }
    }
}

impl From<UpdateUserError> for AppError {
    fn from(error: UpdateUserError) -> Self {
        Self {
            error_type: AppErrorType::UpdateUserError(error),
            msg: None,
        }
    }
}

impl From<UserSessionError> for AppError {
    fn from(error: UserSessionError) -> Self {
        Self {
            error_type: AppErrorType::SessionError(error),
            msg: None,
        }
    }
}

impl From<AuthError> for AppError {
    fn from(error: AuthError) -> Self {
        Self {
            error_type: AppErrorType::AuthError(error),
            msg: None,
        }
    }
}
