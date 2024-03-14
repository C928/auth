use crate::app_error::AppError;
use crate::logic::{ConfirmEmail, Email, FieldValidationError};
use crate::tasks::EmptyGeneratable;
use anyhow::Context;
use chrono::Utc;
use deadpool_redis::redis::{
    from_redis_value, AsyncCommands, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs,
    Value as RedisValue,
};
use deadpool_redis::Connection;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use validator::HasLen;

#[derive(Serialize, Deserialize)]
pub struct URLToken(String);
impl URLToken {
    pub fn parse(token: String) -> Result<Self, FieldValidationError> {
        if token.length() == 150 && token.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Ok(Self { 0: token });
        }

        Err(FieldValidationError::InvalidUrlToken)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn generate() -> Self {
        let rng = thread_rng();
        Self {
            0: rng
                .sample_iter(Alphanumeric)
                .map(char::from)
                .take(150)
                .collect(),
        }
    }

    pub async fn store_user_fields_to_redis(
        mut redis_conn: Connection,
        email: &Email,
    ) -> anyhow::Result<URLToken> {
        loop {
            let token =
                tokio::task::spawn_blocking(move || -> URLToken { URLToken::generate() }).await?;

            let confirmation_fields =
                ConfirmEmail::json_string(email.clone(), Utc::now().timestamp())?;

            if redis_conn
                .hset_nx::<&str, &str, &str, u8>("email", token.as_str(), &confirmation_fields)
                .await?
                != 0
            {
                return Ok(token);
            }
        }
    }

    pub async fn get_associated_redis_fields(
        &self,
        mut redis_conn: Connection,
    ) -> Result<ConfirmEmail, AppError> {
        let fields = match redis_conn
            .hget::<&str, &Self, ConfirmEmail>("email", self)
            .await
        {
            Ok(f) => Ok(f),
            Err(e) => {
                // If the token does not exist in redis, hget will return nil which will result
                // in a TypeError because of string conversion (ConfirmEmail).
                if format!("{:?}", e.kind()) == "TypeError" {
                    Err(FieldValidationError::InvalidUrlToken)?;
                }

                Err(e)
            }
        }
        .with_context(|| "Failed retrieving confirmation fields from redis")?;

        // Removing (token -> fields) entry to make sure the email isn't validated multiple times
        // (user clicking confirmation link multiple times).
        redis_conn
            .hdel("email", &self)
            .await
            .with_context(|| "Failed removing confirmation fields from redis")?;

        Ok(fields)
    }

    pub fn from_str_no_validate(token: String) -> Self {
        Self { 0: token }
    }
}

impl EmptyGeneratable for URLToken {
    fn generate_empty() -> Self {
        Self { 0: "".into() }
    }
}

impl FromRedisValue for URLToken {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        Ok(URLToken::from_str_no_validate(v))
    }
}

impl ToRedisArgs for URLToken {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(self.0.as_bytes());
    }
}

/// Used for token vector allocation
impl Clone for URLToken {
    fn clone(&self) -> Self {
        Self::generate_empty()
    }
}
