use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

pub fn see_other_303(path: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, path))
        .finish()
}

#[macro_export]
macro_rules! ok_400 {
    () => {
        Ok(HttpResponse::Ok().finish())
    };
}
