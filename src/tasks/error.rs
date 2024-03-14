use deadpool_redis::redis::RedisError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Task1Error {
    #[error("{hash_name:} fields async iterator acquisition failed")]
    RedisIterError { err: RedisError, hash_name: String },
    #[error("Removing {hash_name:} fields from redis failed")]
    RedisRemovalError { err: RedisError, hash_name: String },
    #[error("Not all {hash_name:} fields were removed from redis")]
    Removal { hash_name: String },
    #[error("Invalid redis hash name chosen for field deletion")]
    InvalidHashName,
}
