use crate::tasks::error::Task1Error;
use chrono::Utc;
use deadpool_redis::redis::aio::MultiplexedConnection;
use deadpool_redis::redis::{AsyncCommands, AsyncIter, FromRedisValue, ToRedisArgs};
use std::time::Duration;
use tokio::time::sleep;

pub struct Task1Config<'a> {
    redis_conn: MultiplexedConnection,
    hash_name: &'a str,
    expiry_time: u64,
    deletion_bulk_count: usize,
}

impl<'a> Task1Config<'a> {
    pub fn new(
        redis_conn: MultiplexedConnection,
        hash_name: &'a str,
        expiry_time: u64,
        deletion_bulk_count: usize,
    ) -> Self {
        Self {
            redis_conn,
            hash_name,
            expiry_time,
            deletion_bulk_count,
        }
    }
}

/// Used for id vector allocation
pub trait EmptyGeneratable {
    fn generate_empty() -> Self;
}

pub trait Timestampable {
    fn get_timestamp(&self) -> i64;
}

#[tracing::instrument(skip_all)]
pub async fn redis_fields_deletion_task<'a, T, F>(
    mut cfg: Task1Config<'a>,
) -> anyhow::Result<(), Task1Error>
where
    T: EmptyGeneratable + FromRedisValue + ToRedisArgs + Sync + Clone,
    F: Timestampable + FromRedisValue,
    (T, F): Send + Unpin,
{
    let empty_token = T::generate_empty();
    let mut id_vec: Vec<T> = vec![empty_token; cfg.deletion_bulk_count];
    let mut redis_conn_c = cfg.redis_conn.clone();
    loop {
        let timestamp = Utc::now().timestamp();
        let mut iter: AsyncIter<(T, F)> =
            cfg.redis_conn
                .hscan(&cfg.hash_name)
                .await
                .map_err(|e| Task1Error::RedisIterError {
                    err: e,
                    hash_name: cfg.hash_name.into(),
                })?;

        let mut id_index = 0;
        while let Some((id, fields)) = iter.next_item().await {
            if timestamp > cfg.expiry_time as i64 + fields.get_timestamp() {
                id_vec[id_index] = id;
                id_index += 1;
            }

            // Remove fields by bulk for efficiency purpose
            if id_index == cfg.deletion_bulk_count {
                let ret = redis_conn_c
                    .hdel::<&str, &Vec<T>, usize>(&cfg.hash_name, &id_vec)
                    .await
                    .map_err(|e| Task1Error::RedisRemovalError {
                        err: e,
                        hash_name: cfg.hash_name.into(),
                    })?;

                if ret != cfg.deletion_bulk_count {
                    Err(Task1Error::Removal {
                        hash_name: cfg.hash_name.into(),
                    })?;
                }

                tracing::info!("{ret} fields removed from {} hash", cfg.hash_name);
                id_index = 0;
            }
        }

        // Remove remaining fields whose id are in the vector (from index 0 to id_index - 1)
        if id_index != 0 {
            let ret = redis_conn_c
                .hdel::<&str, &[T], usize>(&cfg.hash_name, &id_vec[0..id_index])
                .await
                .map_err(|e| Task1Error::RedisRemovalError {
                    err: e,
                    hash_name: cfg.hash_name.into(),
                })?;

            if ret != id_index {
                Err(Task1Error::Removal {
                    hash_name: cfg.hash_name.into(),
                })?;
            }

            tracing::info!("{ret} fields removed from {} hash", cfg.hash_name);
        }

        sleep(Duration::from_secs(cfg.expiry_time)).await;
    }
}
