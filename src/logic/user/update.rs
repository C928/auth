use crate::app_error::AppError;
use crate::logic::{Email, Password, PasswordHash, Username};
use crate::routes::UpdateUserForm;
use crate::session::UserSessionError;
use secrecy::ExposeSecret;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateUserError {
    InvalidForm,
    InvalidConfirmationSentence,
    UsernameTaken,
    EmailTaken,
    InvalidPassword,
}

pub struct UpdateUser {
    pub password: Password,
    pub new_email: Option<Email>,
    pub new_username: Option<Username>,
    pub new_password: Option<Password>,
}

impl UpdateUser {
    pub fn validate_update_form(form: UpdateUserForm) -> Result<Self, AppError> {
        let password = Password::parse(form.password)?;
        let mut fields_update = [false, false, false];
        let new_email = match form.new_email {
            Some(e) => {
                fields_update[0] = true;
                Some(Email::parse(e)?)
            }
            None => None,
        };

        let new_username = match form.new_username {
            Some(u) => {
                fields_update[1] = true;
                Some(Username::parse(u)?)
            }
            None => None,
        };

        let new_password = match form.new_password {
            Some(p) => match form.new_password_confirm {
                Some(p_confirm) => {
                    if p.expose_secret() != p_confirm.expose_secret() {
                        Err(UpdateUserError::InvalidForm)?;
                    }
                    fields_update[2] = true;
                    Some(Password::parse(p)?)
                }
                None => Err(UpdateUserError::InvalidForm)?,
            },
            None => None,
        };

        if fields_update.iter().all(|x| x == &false) {
            Err(UpdateUserError::InvalidForm)?;
        }

        Ok(Self {
            password,
            new_email,
            new_username,
            new_password,
        })
    }

    pub async fn check_password_is_valid(&self, pool: &PgPool, id: &Uuid) -> Result<(), AppError> {
        let ret = sqlx::query!("select password_hash from users where id = $1", id,)
            .fetch_optional(pool)
            .await?;

        match ret {
            Some(infos) => {
                let hash = PasswordHash::from_str(infos.password_hash);
                self.password.verify_password(&hash)?;
                Ok(())
            }
            None => Err(UserSessionError::InvalidSessionCookie)?,
        }
    }

    pub async fn update_user_in_db(self, pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        if let Some(new_email) = self.new_email {
            let res = sqlx::query!(
                "update users set email = $1 where id = $2",
                new_email.as_str(),
                id,
            )
            .execute(&mut *transaction)
            .await?;

            if res.rows_affected() != 1 {
                Err(UpdateUserError::EmailTaken)?;
            }
        }

        if let Some(new_username) = self.new_username {
            let res = sqlx::query!(
                "update users set username = $1 where id = $2",
                new_username.as_str(),
                id,
            )
            .execute(&mut *transaction)
            .await?;

            if res.rows_affected() != 1 {
                Err(UpdateUserError::UsernameTaken)?;
            }
        }

        if let Some(new_password) = self.new_password {
            let hash = new_password.generate_argon2_hash()?;
            let res = sqlx::query!(
                "update users set password_hash = $1 where id = $2",
                hash.expose_as_str(),
                id,
            )
            .execute(&mut *transaction)
            .await?;

            if res.rows_affected() != 1 {
                Err(AppError::with_msg("Failed updating user password".into()))?;
            }
        }

        transaction.commit().await?;
        Ok(())
    }
}
