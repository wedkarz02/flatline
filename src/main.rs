use std::io;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "./logs",
        format!("{}.log", env!("CARGO_PKG_NAME")),
    );
    let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_target(true)
        .with_current_span(true)
        .with_span_list(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(non_blocking_file_writer);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(io::stdout);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_PKG_NAME")).into()
            }),
        )
        .with(file_layer)
        .with(console_layer)
        .init();

    tracing::info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    if let Err(e) = flatline::run().await {
        tracing::error!("{}", e);
        panic!("{}", e);
    }
}
