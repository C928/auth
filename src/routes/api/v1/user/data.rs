use crate::app_error::AppError;
use crate::session::{UserData, UserSession};
use actix_web::web::Json;
use actix_web::{get, web};
use sqlx::PgPool;

/// Get user data. Used for client side caching (sessionStorage)
#[get("/data")]
async fn get_user_data(
    session: UserSession,
    pg_pool: web::Data<PgPool>,
) -> Result<Json<UserData>, AppError> {
    let id = session.get_session_id()?;
    let user_data = UserData::get_from_db(&pg_pool, id).await?;

    Ok(Json(user_data))
}
