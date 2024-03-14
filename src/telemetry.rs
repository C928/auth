use tracing::subscriber::set_global_default;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing(level: LevelFilter) -> anyhow::Result<()> {
    let env_filter = EnvFilter::new(level.to_string());
    let formatting_layer = tracing_subscriber::fmt::layer()
        .json()
        .compact()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    //LogTracer::init()?;
    set_global_default(subscriber)?;

    Ok(())
}
