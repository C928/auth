use crate::app_error::AppError;
use crate::db::sqlx_user_insertion_error;
use crate::logic::captcha::CaptchaAnswer;
use crate::logic::{
    CaptchaID, Email, FieldValidationError, Password, PasswordHash, URLToken, Username,
};
use crate::routes::{CreateUserForm, CreateUserRequestForm};
use anyhow::Context;
use secrecy::ExposeSecret;
use serde::Serialize;
use sqlx::PgPool;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateUserError {
    CaptchaGeneration,
}

pub struct CreateUserRequest {
    pub email: Email,
    pub captcha_id: CaptchaID,
    pub captcha_answer: CaptchaAnswer,
}

impl CreateUserRequest {
    pub fn validate_register_request_form(form: CreateUserRequestForm) -> Result<Self, AppError> {
        if form.bzz.as_ref().is_some_and(|x| !x.is_empty()) {
            Err(FieldValidationError::NotABee)?;
        }

        form.try_into()
    }

    pub async fn check_email_taken(&self, pool: &PgPool) -> Result<(), AppError> {
        let ret = sqlx::query!(
            "select exists (select 1 from users where email = $1) as exists",
            self.email.as_str(),
        )
        .fetch_one(&*pool)
        .await?;

        if let Some(e) = ret.exists {
            if e {
                Err(FieldValidationError::EmailTaken)?;
            }
        } else {
            Err(AppError::with_msg(
                "Failed retrieving email existence from postgres".into(),
            ))?;
        }

        Ok(())
    }

    pub async fn send_confirmation_email(&self, token: URLToken) {
        //todo: send confirmation email
        let link = format!(
            "{} | https://127.0.0.1:8443/register?token={}",
            &self.email.as_str(),
            token.as_str()
        );
        println!("{link}");
    }
}

impl TryFrom<CreateUserRequestForm> for CreateUserRequest {
    type Error = AppError;

    fn try_from(form: CreateUserRequestForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: Email::parse(form.email)?,
            captcha_id: CaptchaID::parse(form.captcha_id)?,
            captcha_answer: CaptchaAnswer::parse(form.captcha_answer)?,
        })
    }
}

pub struct CreateUser {
    pub token: URLToken,
    pub username: Username,
    pub password: Password,
}

impl CreateUser {
    pub fn validate_register_form(form: CreateUserForm) -> Result<Self, AppError> {
        //todo: check password length first
        if form.password.expose_secret() != form.password_confirm.expose_secret() {
            Err(FieldValidationError::InvalidPasswordFmt)?;
        }

        form.try_into()
    }

    pub async fn check_username_taken(&self, pool: &PgPool) -> Result<(), AppError> {
        let ret = sqlx::query!(
            "select exists (select 1 from users where lower(username) = lower($1)) as exists",
            self.username.as_str(),
        )
        .fetch_one(&*pool)
        .await?;

        if let Some(u) = ret.exists {
            if u {
                Err(FieldValidationError::UsernameTaken)?;
            }
        } else {
            Err(AppError::with_msg(
                "Failed retrieving username existence from postgres".into(),
            ))?;
        }

        Ok(())
    }

    pub async fn insert_user_infos_to_db(
        self,
        pool: &PgPool,
        email: Email,
    ) -> Result<Uuid, AppError> {
        let password_hash =
            tokio::task::spawn_blocking(move || -> Result<PasswordHash, anyhow::Error> {
                Ok(Password::generate_argon2_hash(&self.password)
                    .with_context(|| "Failed generating password hash")?)
            })
            .await??;

        //todo: check email taken pg error
        let ret = sqlx::query!(
            "insert into users (email, username, password_hash) values ($1, $2, $3) returning id",
            email.as_str(),
            self.username.as_str(),
            password_hash.expose_as_str(),
        )
        .fetch_one(pool)
        .await
        .map_err(sqlx_user_insertion_error)?;

        Ok(ret.id)
    }
}

impl TryFrom<CreateUserForm> for CreateUser {
    type Error = AppError;

    fn try_from(form: CreateUserForm) -> Result<Self, Self::Error> {
        Ok(Self {
            token: URLToken::parse(form.token)?,
            username: Username::parse(form.username)?,
            password: Password::parse(form.password)?,
        })
    }
}
