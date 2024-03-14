use anyhow::Context;
use deadpool_redis::{Connection, Pool};

pub async fn get_redis_connection(pool: &Pool) -> anyhow::Result<Connection> {
    pool.get()
        .await
        .with_context(|| "Failed retrieving a redis connection from the pool")
}

//pub const REDIS_VERIFY_KEY: &'static str = "verify";
//pub const REDIS_PWD_RESET_KEY: &'static str = "pwd-reset";
