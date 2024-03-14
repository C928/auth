use actix_web::web::Data;
use auth::config::Settings;
use auth::server::{start_server, ServerSetup};
use auth::telemetry::init_tracing;
use deadpool_redis::Pool as RedisPool;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::level_filters::LevelFilter;

static SETTINGS_WITH_LOGS: Lazy<Settings> = Lazy::new(|| {
    if std::env::var("LOGS").is_ok() {
        init_tracing(LevelFilter::INFO).unwrap();
    } else {
        init_tracing(LevelFilter::OFF).unwrap();
    }

    Settings::new("config/test").unwrap()
});

pub struct ApiTestUtils {
    pub redis_pool: Data<RedisPool>,
    pub pg_pool: Data<PgPool>,
    pub http_client: reqwest::Client,
}

impl From<&ServerSetup> for ApiTestUtils {
    fn from(value: &ServerSetup) -> Self {
        Self {
            redis_pool: value.redis_pool.clone(),
            pg_pool: value.pg_pool.clone(),
            http_client: reqwest::Client::new(),
        }
    }
}

pub async fn start_test_server() -> ApiTestUtils {
    let settings = Lazy::force(&SETTINGS_WITH_LOGS);
    let setup = ServerSetup::new(&settings).await.unwrap();

    let test_utils = ApiTestUtils::from(&setup);
    sqlx::migrate!("./migrations")
        .run(&**setup.pg_pool)
        .await
        .unwrap();

    let _ = tokio::spawn(start_server(settings.clone(), setup));
    // give time for the server to start
    sleep(Duration::from_secs(1)).await;

    test_utils
}
