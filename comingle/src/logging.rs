use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging(log_level: &str, pretty_log: bool) {
    let env_filter = EnvFilter::new(log_level);

    let fmt_layer = if pretty_log {
        tracing_subscriber::fmt::layer()
            .pretty()
            .compact()
            .with_file(false)
            .with_line_number(false)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer().json().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
