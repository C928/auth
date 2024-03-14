use crate::app_error::AppError;
use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use anyhow::Context;
use serde::Serialize;
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSessionError {
    InvalidSessionCookie,
}

pub struct UserSession(Session);
impl UserSession {
    pub fn is_active(&self) -> Result<bool, SessionGetError> {
        self.0
            .get("id")?
            .map(|_: String| Ok(true))
            .unwrap_or(Ok(false))
    }

    pub fn activate(&self, id: Uuid) -> Result<(), SessionInsertError> {
        self.0.renew();
        self.0
            .insert("id", id)
            .with_context(|| "Failed activating user session")?;
        Ok(())
    }

    pub fn deactivate(&self) {
        self.0.purge();
    }

    pub fn get_session_id(&self) -> Result<Uuid, AppError> {
        match self.0.get("id")? {
            Some(id) => Ok(id),
            None => Err(UserSessionError::InvalidSessionCookie)?,
        }
    }
}

impl FromRequest for UserSession {
    type Error = Error;
    type Future = Ready<Result<UserSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(UserSession(req.get_session())))
    }
}
