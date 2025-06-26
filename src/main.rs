use std::{io, path::PathBuf};

use clap::Parser;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// TODO: Maybe add a '--init [Optional: PATH]' option that creates a .flatline/ directory
//       in PATH (or home dir by default) and creates both config.json template and logs/
//       directory there. Use the PATH for logs in tracing file appender.

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Optional path to a configuration JSON file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.config.is_none() {
        dotenvy::dotenv().ok();
    }

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "./logs",
        format!("{}.log", env!("CARGO_PKG_NAME")),
    );
    let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_target(false)
        .with_current_span(true)
        .with_span_list(false)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(non_blocking_file_writer);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true)
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

    if let Err(e) = flatline::run(args.config).await {
        tracing::error!("{}", e);
        panic!("{}", e);
    }
}
