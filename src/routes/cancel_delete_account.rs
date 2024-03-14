use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse};

#[get("/delete-account/cancel")]
pub async fn get_account_delete_cancel_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../../static/html/cancel-delete-account.html"))
}
