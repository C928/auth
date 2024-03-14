use auth::app_error::select_return;
use auth::config::Settings;
use auth::server::{start_redis_fields_deletion_task, start_server, ServerSetup};
use auth::telemetry::init_tracing;
use tracing::level_filters::LevelFilter;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(LevelFilter::INFO)?;

    let settings = Settings::new("config/dev")?;
    let setup = ServerSetup::new(&settings).await?;

    let task1_email = tokio::spawn(start_redis_fields_deletion_task(settings.clone(), "email"));
    let task1_captcha = tokio::spawn(start_redis_fields_deletion_task(settings.clone(), "captcha"));
    let srv = tokio::spawn(start_server(settings, setup));

    tokio::select! {
        ret = srv => select_return("server", ret),
        ret = task1_email => select_return("task1 (redis deletion: email confirm)", ret),
        ret = task1_captcha => select_return("task1 (redis deletion: captcha)", ret),
    }

    Ok(())
}


//todo: first ==============================
//todo: confirm email for deletion and email change
//todo: send email on account update and delete
//todo: task2_user_del background task for pg deletion + send email after successful deletion
//todo: use POST instead of GET for /logout

//todo: impl sqlx::ToRow for email, username, ... + rm as_str() in queries
//todo: schema folder for SQLXUser, ...
//todo: replace fetch_optionals with fetch_one where possible
//todo: add transactions where needed
//todo: use sqlx query_as macro where possible along with tuple struct (#derive(sqlx::FromRow))
//  or standard struct

//todo: api tests
//todo: ci

//todo: task1 error if new entries appended
//todo: log line number on unknown errors
//todo: flush redis before starting server (removes old sessions) + lifetime for sessions in redis
//todo: HttpsRedirect middleware not working with bind_rustls_021

//todo: frontend ===========================
//todo: port js to ts (+ use ts-rs for response types)
//todo: replace .err with .log
//todo: catch js fetch errors
//todo: css
