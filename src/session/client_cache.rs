use crate::app_error::AppError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_with::TimestampSeconds;
use sqlx::PgPool;
use uuid::Uuid;

#[serde_with::serde_as]
#[derive(Serialize)]
pub struct UserData {
    email: String,
    username: String,
    bio: Option<String>,
    #[serde_as(as = "TimestampSeconds<String>")]
    registration_date: DateTime<Utc>,
}

impl UserData {
    pub async fn get_from_db(pool: &PgPool, id: Uuid) -> Result<Self, AppError> {
        let user_data = sqlx::query_as!(
            UserData,
            "select email, username, bio, registration_date from users where id = $1",
            id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user_data)
    }
}
