use crate::utils::start_task1;
use auth::logic::{CaptchaAnswer, CaptchaFields, CaptchaID, ConfirmEmail, Email, URLToken};
use chrono::Utc;
use deadpool_redis::redis::{Commands, ToRedisArgs};
use fake::faker::internet::en::SafeEmail;
use fake::Fake;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

#[test]
fn email_confirmation_entries_are_successfully_removed_from_redis() {
    redis_fields_deletion::<ConfirmEmailEntry>("email");
}

#[test]
fn captcha_entries_are_successfully_removed_from_redis() {
    redis_fields_deletion::<CaptchaEntry>("captcha");
}

fn redis_fields_deletion<G>(hash_name: &str)
where
    G: RedisEntryGenerator,
{
    let (tx, rx) = oneshot::channel::<()>();
    let (mut cfg, _rt) = start_task1(rx, hash_name.into());

    // 1 is subtracted to cover the case where 'number of entries' < deletion_bulk_count
    let insertion_cnt = cfg.deletion_bulk_count * 3 - 1;
    let mut entries = Vec::with_capacity(insertion_cnt);
    for _ in 0..insertion_cnt {
        let (id, fields) = G::generate_random_entry();
        entries.push((id, fields));
    }

    cfg.redis_conn
        .hset_multiple::<&str, G::Key, String, bool>(hash_name, &entries)
        .unwrap();
    let inserted_cnt: usize = cfg.redis_conn.hlen(hash_name).unwrap();
    assert_eq!(inserted_cnt, insertion_cnt);

    // Test that entries have been inserted. We then send a signal to start task1 (deletion task)
    tx.send(()).unwrap();
    sleep(Duration::from_secs(cfg.expiry_time) * 3);

    // Task1 should have removed deletion_bulk_count + deletion_bulk_count + leftovers.
    // 0 entry should then be left in redis.
    let entries_cnt: usize = cfg.redis_conn.hlen(hash_name).unwrap();
    assert_eq!(entries_cnt, 0);
}

trait RedisEntryGenerator {
    type Key: ToRedisArgs;

    fn generate_random_entry() -> (Self::Key, String);
}

struct ConfirmEmailEntry;
impl RedisEntryGenerator for ConfirmEmailEntry {
    type Key = URLToken;

    fn generate_random_entry() -> (Self::Key, String) {
        let timestamp = Utc::now().timestamp();
        let email = Email::parse(SafeEmail().fake()).unwrap();
        let fields = ConfirmEmail::json_string(email, timestamp).unwrap();
        let token = URLToken::generate();
        (token, fields)
    }
}

struct CaptchaEntry;
impl RedisEntryGenerator for CaptchaEntry {
    type Key = CaptchaID;

    fn generate_random_entry() -> (Self::Key, String) {
        let timestamp = Utc::now().timestamp();
        let captcha_answer = CaptchaAnswer::parse((4..=6).fake()).unwrap();
        let fields = CaptchaFields::json_string(captcha_answer, timestamp).unwrap();

        let id = CaptchaID::from_uuid(Uuid::new_v4());
        (id, fields)
    }
}
