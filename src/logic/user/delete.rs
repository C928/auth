use crate::app_error::AppError;
use crate::logic::{Email, FieldValidationError, Password, PasswordHash, URLToken, Username};
use crate::routes::DeleteUserRequestForm;
use crate::session::UserSessionError;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

pub struct DeleteUserRequest {
    pub id: Uuid,
    pub password: Password,
}

#[derive(sqlx::FromRow)]
pub struct SQLXUser {
    email: Email,
    username: Username,
}

impl DeleteUserRequest {
    pub fn validate_delete_user_request_form(
        form: DeleteUserRequestForm,
        id: Uuid,
    ) -> Result<Self, FieldValidationError> {
        Ok(Self {
            id,
            password: Password::parse(form.password)?,
        })
    }

    pub async fn verify_password(&self, pool: &PgPool) -> Result<(), AppError> {
        let ret = query!("select password_hash from users where id = $1", self.id)
            .fetch_optional(pool)
            .await?;

        if let Some(ret) = ret {
            self.password
                .verify_password(&PasswordHash::from_str(ret.password_hash))?;
        } else {
            Err(UserSessionError::InvalidSessionCookie)?;
        }

        Ok(())
    }

    pub async fn insert_account_deletion_entry_to_db(
        &self,
        pool: &PgPool,
    ) -> Result<(URLToken, SQLXUser), AppError> {
        let token =
            tokio::task::spawn_blocking(move || -> URLToken { URLToken::generate() }).await?;

        let mut transaction = pool.begin().await?;
        query!(
            "insert into account_deletions (id, account_id) values ($1, $2)",
            token.as_str(),
            self.id
        )
        .execute(&mut *transaction)
        .await?;

        query!(
            "update users set requested_deletion = true where id = $1",
            self.id
        )
        .execute(&mut *transaction)
        .await?;

        let user = query_as!(
            SQLXUser,
            "select email, username from users where id = $1",
            self.id
        )
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok((token, user))
    }

    pub async fn send_account_deletion_requested_email(cancel_token: URLToken, user: SQLXUser) {
        //todo: send email
        let link = format!(
            "Hi {},\nWe have received a request to delete your account.\n\
            All data associated with this account will be removed in 15 days.\n\n\
            If you are not the author of this request please log into your account or click here\n\
            to the cancel your account deletion. Also, change your password as soon as you can\n\
            because someone knows it!\n\
            {} | https://127.0.0.1:8443/delete-account/cancel?token={}",
            user.username.as_str(),
            user.email.as_str(),
            cancel_token.as_str()
        );
        println!("{link}");
    }
}

pub struct CancelUserDeletion {
    token: URLToken,
}

impl CancelUserDeletion {
    pub fn from_url_token(token: String) -> Result<Self, AppError> {
        Ok(Self {
            token: URLToken::parse(token)?,
        })
    }

    pub async fn remove_deletion_fields_with_token(&self, pool: &PgPool) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        let ret = query!(
            "delete from account_deletions where id = $1 returning account_id",
            self.token.as_str()
        )
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(rec) = ret {
            query!(
                "update users set requested_deletion = null where id = $1",
                rec.account_id
            )
            .execute(&mut *transaction)
            .await?;
        } else {
            Err(FieldValidationError::InvalidUrlToken)?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn remove_deletion_fields_with_user_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut transaction = pool.begin().await?;
        query!("delete from account_deletions where account_id = $1", id)
            .execute(&mut *transaction)
            .await?;

        query!(
            "update users set requested_deletion = null where id = $1",
            id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}
