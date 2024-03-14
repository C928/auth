use crate::logic::Email;
use crate::tasks::Timestampable;
use deadpool_redis::redis::{
    from_redis_value, ErrorKind, FromRedisValue, RedisResult, Value as RedisValue,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ConfirmEmail {
    pub email: Email,
    pub timestamp: i64,
}

impl ConfirmEmail {
    pub fn json_string(email: Email, timestamp: i64) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string(&Self { email, timestamp })?)
    }
}

impl Timestampable for ConfirmEmail {
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl FromRedisValue for ConfirmEmail {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        match serde_json::from_str(&v) {
            Ok(f) => Ok(f),
            Err(_) => Err((
                ErrorKind::TypeError,
                "deserializing email confirmation fields",
            )
                .into()),
        }
    }
}
