use crate::app_error::AppError;
use crate::logic::{CreateUserError, FieldValidationError};
use crate::tasks::{EmptyGeneratable, Timestampable};
use captcha::{gen, Difficulty};
use chrono::Utc;
use deadpool_redis::redis::{
    from_redis_value, AsyncCommands, ErrorKind, FromRedisValue, RedisResult, RedisWrite,
    ToRedisArgs, Value as RedisValue,
};
use deadpool_redis::Connection;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
//use base64::Engine;
use uuid::Uuid;

pub struct Captcha {
    id: CaptchaID,
    answer: CaptchaAnswer,
    img: String,
    //wav: String,
}

impl Captcha {
    pub fn generate() -> Result<Self, AppError> {
        let captcha = gen(Difficulty::Hard);
        let img = match captcha.as_base64() {
            Some(b) => b,
            None => Err(CreateUserError::CaptchaGeneration)?,
        };

        /*
        let wav = captcha.as_wav();
        let mut wav_data: Vec<u8> = vec![];
        for d in wav.iter() {
            if let Some(data) = d {
                wav_data.extend(data);
            }
        }
        let wav = base64::engine::general_purpose::STANDARD.encode(&wav_data);
        */
        let answer = CaptchaAnswer::from_str(captcha.chars_as_string());
        Ok(Self {
            id: CaptchaID::from_uuid(Uuid::new_v4()),
            answer,
            img,
            //wav,
        })
    }

    pub async fn reload_captcha(
        id: CaptchaID,
        redis_conn: &mut Connection,
    ) -> Result<Self, AppError> {
        let existed: bool = redis_conn.hdel("captcha", id).await?;
        if !existed {
            Err(FieldValidationError::InvalidCaptchaID)?;
        }

        Self::generate()
    }

    pub async fn store_captcha_answer_in_redis(
        &self,
        mut redis_conn: Connection,
    ) -> Result<(), AppError> {
        let timestamp = Utc::now().timestamp();
        let answer = CaptchaFields::json_string(self.answer.clone(), timestamp)?;
        let res: bool = redis_conn.hset_nx("captcha", &self.id, answer).await?;
        if !res {
            Err(CreateUserError::CaptchaGeneration)?;
        }

        Ok(())
    }

    pub fn get_response_data(self) -> CaptchaResponseData {
        CaptchaResponseData {
            id: self.id,
            img: self.img,
            //wav: self.wav,
        }
    }
}

#[derive(Serialize)]
pub struct CaptchaResponseData {
    id: CaptchaID,
    img: String,
    //wav: String,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub struct CaptchaAnswer(String);
impl CaptchaAnswer {
    pub fn parse(captcha_answer: String) -> Result<Self, FieldValidationError> {
        let len = captcha_answer.len();
        if len < 4 || len > 6 || !captcha_answer.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err(FieldValidationError::InvalidCaptchaAnswer)?;
        }

        Ok(Self { 0: captcha_answer })
    }

    pub async fn is_valid_captcha_answer(
        redis_conn: &mut Connection,
        captcha_answer: &CaptchaAnswer,
        captcha_id: &CaptchaID,
    ) -> Result<(), AppError> {
        let fields: CaptchaFields = redis_conn.hget("captcha", captcha_id).await?;
        if &fields.answer != captcha_answer {
            Err(FieldValidationError::InvalidCaptchaAnswer)?;
        }
        redis_conn.hdel("captcha", &captcha_id).await?;

        Ok(())
    }

    pub fn from_str(captcha_answer: String) -> Self {
        Self { 0: captcha_answer }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize, Serialize)]
pub struct CaptchaFields {
    pub answer: CaptchaAnswer,
    pub timestamp: i64,
}

impl CaptchaFields {
    pub fn json_string(answer: CaptchaAnswer, timestamp: i64) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string(&Self { answer, timestamp })?)
    }
}

impl Timestampable for CaptchaFields {
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl FromRedisValue for CaptchaFields {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        match serde_json::from_str(&v) {
            Ok(f) => Ok(f),
            Err(_) => Err((ErrorKind::TypeError, "deserializing captcha fields").into()),
        }
    }
}

#[derive(Serialize)]
pub struct CaptchaID(String);
impl CaptchaID {
    pub fn parse(captcha_id: String) -> Result<Self, AppError> {
        if !Uuid::from_str(&captcha_id).is_ok() {
            Err(FieldValidationError::InvalidCaptchaID)?;
        }

        Ok(Self { 0: captcha_id })
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self { 0: id.to_string() }
    }

    pub fn from_str(id: String) -> Self {
        Self { 0: id }
    }
}

impl EmptyGeneratable for CaptchaID {
    fn generate_empty() -> Self {
        Self { 0: "".into() }
    }
}

impl FromRedisValue for CaptchaID {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        Ok(CaptchaID::from_str(v))
    }
}

impl ToRedisArgs for CaptchaID {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(self.0.as_bytes());
    }
}

// Used for id vector allocation
impl Clone for CaptchaID {
    fn clone(&self) -> Self {
        Self::generate_empty()
    }
}
