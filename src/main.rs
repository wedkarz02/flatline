use std::{io, path::PathBuf};

use clap::Parser;
use flatline::config::Config;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use valuable::Valuable;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Command {
    /// Initialize flatline configuration in the home directory
    Init,
    /// Run with configuration from specific source
    Config {
        #[command(subcommand)]
        source: ConfigSource,
    },
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum ConfigSource {
    /// Use environment variables for configuration
    Env,
    /// Use JSON file for configuration
    Json {
        /// Path to JSON config file
        path: PathBuf,
    },
}

async fn init_base_dir() -> anyhow::Result<()> {
    let base_dir = dirs::home_dir()
        .expect("home directory should exist")
        .join(format!(".{}", env!("CARGO_PKG_NAME")));

    std::fs::create_dir_all(&base_dir)?;

    let config_path = base_dir.join("config.json");
    if !config_path.exists() {
        let config_template = Config::default();
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&config_template)?,
        )?;
    }

    Ok(())
}

async fn parse_cli(args: Args) -> anyhow::Result<Option<Config>> {
    match args.command {
        Some(Command::Init) => {
            init_base_dir().await?;
            Ok(None)
        }
        Some(Command::Config { source }) => {
            let config = match source {
                ConfigSource::Env => Config::from_env(),
                ConfigSource::Json { path } => Config::from_json(&path)?,
            };
            Ok(Some(config))
        }
        None => {
            let default_config_path = dirs::home_dir()
                .expect("home directory should exist")
                .join(format!(".{}/config.json", env!("CARGO_PKG_NAME")));

            let config = match default_config_path.exists() {
                true => Config::from_json(&default_config_path)?,
                false => Config::from_env(),
            };

            Ok(Some(config))
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let config = match parse_cli(args).await {
        Ok(res) => match res {
            Some(c) => c,
            None => {
                println!("Configuration directory initialized in '~/.{}'. Make sure to fill out the 'config.json' file.", env!("CARGO_PKG_NAME"));
                std::process::exit(0);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            panic!("{}", e);
        }
    };

    let base_dir = dirs::home_dir()
        .expect("home directory should exist")
        .join(format!(".{}", env!("CARGO_PKG_NAME")));

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        if base_dir.exists() {
            base_dir.join("logs")
        } else {
            PathBuf::from("./logs")
        },
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
    tracing::info!(
        config = config.redacted().as_value(),
        "Configuration loaded"
    );

    if let Err(e) = flatline::run(config).await {
        tracing::error!("{}", e);
        panic!("{}", e);
    }
}
