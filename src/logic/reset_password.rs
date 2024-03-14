use crate::app_error::AppError;
use crate::logic::{
    CaptchaAnswer, CaptchaID, Email, FieldValidationError, Password, PasswordHash, URLToken,
    Username,
};
use crate::routes::{ResetPasswordForm, ResetPasswordRequestForm};
use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::{query_as, PgPool};

pub struct ResetPasswordRequest {
    pub email: Email,
    pub captcha_id: CaptchaID,
    pub captcha_answer: CaptchaAnswer,
}
impl ResetPasswordRequest {
    pub fn validate_password_reset_request_form(
        form: ResetPasswordRequestForm,
    ) -> Result<ResetPasswordRequest, AppError> {
        if form.bzz.as_ref().is_some_and(|x| !x.is_empty()) {
            Err(FieldValidationError::NotABee)?;
        }

        form.try_into()
    }

    pub async fn send_confirmation_email(&self, token: URLToken) {
        //todo: send confirmation email
        let link = format!(
            "https://127.0.0.1:8443/reset-password?token={}",
            token.as_str()
        );
        println!("{link}");
    }
}

impl TryFrom<ResetPasswordRequestForm> for ResetPasswordRequest {
    type Error = AppError;

    fn try_from(form: ResetPasswordRequestForm) -> Result<Self, Self::Error> {
        Ok(Self {
            email: Email::parse(form.email)?,
            captcha_id: CaptchaID::parse(form.captcha_id)?,
            captcha_answer: CaptchaAnswer::parse(form.captcha_answer)?,
        })
    }
}

pub struct ResetPassword {
    pub token: URLToken,
    pub new_password: Password,
}

impl ResetPassword {
    pub fn validate_reset_password_form(form: ResetPasswordForm) -> Result<Self, AppError> {
        //todo: check password length first
        if form.new_password.expose_secret() != form.new_password_confirm.expose_secret() {
            Err(FieldValidationError::InvalidPasswordFmt)?;
        }

        form.try_into()
    }

    pub async fn update_password_in_db(
        self,
        pool: &PgPool,
        email: Email,
    ) -> Result<PasswordUpdated, AppError> {
        let new_password_hash =
            tokio::task::spawn_blocking(move || -> Result<PasswordHash, anyhow::Error> {
                let hash = Password::generate_argon2_hash(&self.new_password)
                    .with_context(|| "Failed generating password hash")?;
                Ok(hash)
            })
            .await??;

        #[derive(sqlx::FromRow)]
        struct UsernameRow {
            username: Username,
        }

        let username: UsernameRow = query_as!(
            UsernameRow,
            "update users set password_hash = $1 where email = $2 returning username",
            new_password_hash.expose_as_str(),
            email.as_str(),
        )
        .fetch_one(pool)
        .await?;

        Ok(PasswordUpdated {
            email,
            username: username.username,
        })
    }
}

impl TryFrom<ResetPasswordForm> for ResetPassword {
    type Error = AppError;

    fn try_from(form: ResetPasswordForm) -> Result<Self, Self::Error> {
        Ok(Self {
            token: URLToken::parse(form.token)?,
            new_password: Password::parse(form.new_password)?,
        })
    }
}

pub struct PasswordUpdated {
    email: Email,
    username: Username,
}
impl PasswordUpdated {
    pub async fn send_password_updated_email(self) {
        //todo: send password updated email
        println!(
            "{} | Hi {},\nYour password has been updated as you requested\n\
        If you are not the author of this request, please contact our support as soon as possible\n",
            self.email.as_str(),
            self.username.as_str(),
        );
    }
}
