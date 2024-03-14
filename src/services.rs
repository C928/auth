use crate::routes::{
    cancel_delete_user_request, create_user, create_user_request, delete_user_request,
    get_account_delete_cancel_page, get_home_page, get_login_page, get_register_page,
    get_register_request_page, get_reset_password_page, get_reset_password_request_page,
    get_settings_page, get_user_data, load_captcha, login_user, logout_user, reload_captcha,
    reset_user_password, reset_user_password_request, update_user,
};
use actix_files::Files;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse};

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_home_page)
        .service(get_login_page)
        .service(get_register_request_page)
        .service(get_register_page)
        .service(get_settings_page)
        .service(get_reset_password_request_page)
        .service(get_reset_password_page)
        .service(get_account_delete_cancel_page)
        .service(
            web::scope("/api/v1")
                .service(
                    web::scope("/user")
                        .service(create_user_request)
                        .service(create_user)
                        .service(delete_user_request)
                        .service(cancel_delete_user_request)
                        .service(update_user)
                        .service(get_user_data)
                        .service(login_user)
                        .service(logout_user),
                )
                .service(load_captcha)
                .service(reload_captcha)
                .service(reset_user_password_request)
                .service(reset_user_password),
        )
        .service(Files::new("/", "static").index_file("index.html"))
        .default_service(web::to(|| async {
            HttpResponse::NotFound()
                .content_type(ContentType::html())
                .body(include_str!("../static/html/errors/404.html"))
        }));
}
