use crate::app_error::AppError;
use crate::db::get_redis_connection;
use crate::logic::{Captcha, CaptchaID, CaptchaResponseData};
use actix_web::web::Json;
use actix_web::{get, web};
use deadpool_redis::Pool as RedisPool;
use serde::Deserialize;

#[get("/captcha")]
async fn load_captcha(
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<CaptchaResponseData>, AppError> {
    let captcha = Captcha::generate()?;
    let redis_conn = get_redis_connection(&redis_pool).await?;
    captcha.store_captcha_answer_in_redis(redis_conn).await?;

    let data = captcha.get_response_data();
    Ok(Json(data))
}

#[derive(Deserialize)]
pub struct CaptchaIDParam {
    id: String,
}

#[get("/captcha/reload")]
async fn reload_captcha(
    web::Query(param): web::Query<CaptchaIDParam>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Json<CaptchaResponseData>, AppError> {
    let id = CaptchaID::parse(param.id)?;

    let mut redis_conn = get_redis_connection(&redis_pool).await?;
    let new_captcha = Captcha::reload_captcha(id, &mut redis_conn).await?;
    new_captcha
        .store_captcha_answer_in_redis(redis_conn)
        .await?;

    let data = new_captcha.get_response_data();
    Ok(Json(data))
}
