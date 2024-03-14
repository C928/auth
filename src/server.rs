use crate::config::Settings;
use crate::logic::{CaptchaFields, CaptchaID, ConfirmEmail, URLToken};
use crate::services::services;
use crate::tasks::{redis_fields_deletion_task, Task1Config, Task1Error};
use actix_cors::Cors;
use actix_governor::governor::middleware::NoOpMiddleware;
use actix_governor::{Governor, GovernorConfig, GovernorConfigBuilder, PeerIpKeyExtractor};
use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::SameSite;
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{cookie, App, HttpServer};
use actix_web_lab::middleware::RedirectHttps;
use anyhow::{anyhow, bail};
use deadpool_redis::redis::{Client, ConnectionLike};
use deadpool_redis::{Config, Pool as RedisPool, Runtime};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

pub async fn start_server(settings: Settings, setup: ServerSetup) -> anyhow::Result<()> {
    let host = settings.application_host.clone();
    let srv = HttpServer::new(move || {
        App::new()
            .wrap(RedirectHttps::default().to_port(8443))
            .wrap(
                SessionMiddleware::builder(setup.session_store.clone(), setup.session_pkey.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(SameSite::Strict)
                    .cookie_content_security(CookieContentSecurity::Private)
                    .session_lifecycle(BrowserSession::default())
                    .build(),
            )
            .wrap(
                Cors::default()
                    .allowed_origin(&format!("https://{}:{}", host, settings.application_port))
                    .allowed_methods(vec!["GET", "POST", "PUT", "OPTIONS"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .wrap(Governor::new(&setup.governor_config))
            .wrap(TracingLogger::default())
            .app_data(setup.redis_pool.clone())
            .app_data(setup.pg_pool.clone())
            .configure(services)
    })
    .bind_rustls_021(
        (settings.application_host, settings.application_port),
        setup.rustls_config,
    )?
    .run();

    srv.await?;
    Ok(())
}



pub struct ServerSetup {
    pub redis_pool: Data<RedisPool>,
    pub pg_pool: Data<PgPool>,
    pub governor_config: GovernorConfig<PeerIpKeyExtractor, NoOpMiddleware>,
    pub session_store: RedisSessionStore,
    pub session_pkey: cookie::Key,
    pub rustls_config: rustls::ServerConfig,
}

impl ServerSetup {
    pub async fn new(settings: &Settings) -> anyhow::Result<Self> {
        let pg_pool = match PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy(settings.postgres.url.expose_secret())
        {
            Ok(p) => Data::new(p),
            Err(_) => bail!("Postgres instance hasn't been started"),
        };

        let cfg = Config::from_url(&settings.redis.url);
        let redis_pool = Data::new(cfg.create_pool(Some(Runtime::Tokio1))?);

        let governor_config = GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(20)
            .finish()
            .ok_or(anyhow!("Getting rate limiter governor configuration"))?;

        let session_store = RedisSessionStore::new(&settings.redis.url).await?;
        let session_pkey = cookie::Key::generate();

        let rustls_config = Self::load_rustls_config()?;

        Ok(Self {
            redis_pool,
            pg_pool,
            governor_config,
            session_store,
            session_pkey,
            rustls_config,
        })
    }

    fn load_rustls_config() -> anyhow::Result<rustls::ServerConfig> {
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        let cert_file = &mut BufReader::new(File::open("tls/cert.pem")?);
        let key_file = &mut BufReader::new(File::open("tls/key.pem")?);

        let cert_chain = certs(cert_file)?.into_iter().map(Certificate).collect();

        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)?
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            bail!("Could not locate PKCS 8 private keys.");
        }

        Ok(config.with_single_cert(cert_chain, keys.remove(0))?)
    }
}

pub async fn start_redis_fields_deletion_task(settings: Settings, hash_name: &str) -> anyhow::Result<()> {
    let redis_client = Client::open(&*settings.redis.url)?;
    if !redis_client.is_open() {
        bail!(
            "Redis instance hasn't been started on {}",
            settings.redis.url
        );
    }

    let task_redis_conn = redis_client.get_multiplexed_tokio_connection().await?;
    match hash_name {
        "email" => {
            let task1_cfg = Task1Config::new(
                task_redis_conn,
                hash_name,
                settings.task1_email_confirm.expiry_time,
                settings.task1_email_confirm.deletion_bulk_count,
            );
            redis_fields_deletion_task::<URLToken, ConfirmEmail>(task1_cfg).await?;
        }
        "captcha" => {
            let task1_cfg = Task1Config::new(
                task_redis_conn,
                hash_name,
                settings.task1_captcha.expiry_time,
                settings.task1_captcha.deletion_bulk_count,
            );
            redis_fields_deletion_task::<CaptchaID, CaptchaFields>(task1_cfg).await?;
        }
        _ => Err(Task1Error::InvalidHashName)?,
    }

    Ok(())
}

/*
pub async fn start_pg_accounts_deletion_task(settings: Settings) -> anyhow::Result<()> {

}
 */
