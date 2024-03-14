use crate::app_error::AppError;
use crate::session::UserSession;
use actix_web::http::header::LOCATION;
use actix_web::{get, HttpResponse};

#[tracing::instrument(skip_all)]
#[get("/logout")]
pub async fn logout_user(session: UserSession) -> Result<HttpResponse, AppError> {
    if session.is_active()? {
        session.deactivate();
    }

    Ok(HttpResponse::NoContent()
        .insert_header((LOCATION, "/login"))
        .finish())
}
