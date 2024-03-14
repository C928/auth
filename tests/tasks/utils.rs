use auth::config::Settings;
use auth::server::start_fields_deletion_task;
use deadpool_redis::redis;
use deadpool_redis::redis::{Client, Connection};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

pub struct Task1TestSettings {
    pub redis_conn: Connection,
    pub expiry_time: u64,
    pub deletion_bulk_count: usize,
}

impl Task1TestSettings {
    fn get_settings_from_hash(settings: &Settings, hash_name: &str) -> Self {
        let client = Client::open(&*settings.redis.url).unwrap();
        let (expiry_time, deletion_bulk_count) = match hash_name {
            "email" => (
                settings.task1_email_confirm.expiry_time,
                settings.task1_email_confirm.deletion_bulk_count,
            ),
            "captcha" => (
                settings.task1_captcha.expiry_time,
                settings.task1_captcha.deletion_bulk_count,
            ),
            _ => panic!("invalid hash name"),
        };

        Self {
            redis_conn: client.get_connection().unwrap(),
            expiry_time,
            deletion_bulk_count,
        }
    }
}

pub fn start_task1(rx: oneshot::Receiver<()>, hash_name: String) -> (Task1TestSettings, Runtime) {
    let settings = Settings::new("config/test").unwrap();
    let mut test_utils = Task1TestSettings::get_settings_from_hash(&settings, &hash_name);
    let _: bool = redis::cmd("FLUSHDB")
        .query(&mut test_utils.redis_conn)
        .unwrap();

    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        rx.await.unwrap();
        start_fields_deletion_task(settings, &hash_name)
            .await
            .unwrap();
    });

    (test_utils, rt)
}
