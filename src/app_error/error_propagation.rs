use tokio::task::JoinError;
use tracing::{error, info};

fn error_format<E>(error: E, origin: &str)
where
    E: std::fmt::Debug + std::fmt::Display,
{
    error!(
        error.debug = ?error,
        error.display = %error,
        "{} returned with app_error",
        origin,
    );
}

pub fn select_return(origin: &str, ret: Result<Result<(), anyhow::Error>, JoinError>) {
    match ret {
        Ok(Ok(_)) => info!("{} successfully returned", origin),
        Ok(Err(e)) => error_format(e, origin),
        Err(e) => error_format(e, origin),
    }
}
