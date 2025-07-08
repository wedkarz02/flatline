use std::{io, path::PathBuf};

use clap::Parser;
use flatline::{
    config::Config,
    init_database,
    models::user::{Role, User},
    services::auth::hash_string,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use valuable::Valuable;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, clap::Subcommand)]
enum Command {
    /// Initialize flatline configuration in the home directory
    Init,
    /// Run with configuration from specific source
    Config {
        #[command(subcommand)]
        source: ConfigSource,
    },
    /// Execute administrative commands
    Exec {
        #[command(subcommand)]
        command: ExecCommand,
    },
}

#[derive(Clone, Debug, clap::Subcommand)]
enum ConfigSource {
    /// Use environment variables for configuration
    Env,
    /// Use JSON file for configuration
    Json {
        /// Path to JSON config file
        path: PathBuf,
    },
}

#[derive(Clone, Debug, clap::Subcommand)]
enum ExecCommand {
    /// Create an admin user
    CreateAdmin {
        /// Username for the admin user
        #[arg(short, long)]
        username: String,
        /// Password for the admin user
        #[arg(short, long)]
        password: String,
    },
}

impl ExecCommand {
    async fn handle_exec_command(&self, config: Config) -> anyhow::Result<()> {
        match self {
            ExecCommand::CreateAdmin { username, password } => {
                let admin_user = User::new(
                    username,
                    &hash_string(&password),
                    &[Role::User, Role::Admin],
                );

                let db = init_database(&config).await?;
                db.users().create(admin_user).await?;

                Ok(())
            }
        }
    }
}

fn init_base_dir() -> anyhow::Result<()> {
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

fn choose_config() -> anyhow::Result<Config> {
    let default_config_path = dirs::home_dir()
        .expect("home directory should exist")
        .join(format!(".{}/config.json", env!("CARGO_PKG_NAME")));

    let config = match default_config_path.exists() {
        true => Config::from_json(&default_config_path)?,
        false => Config::from_env(),
    };

    Ok(config)
}

async fn parse_cli(args: Args) -> anyhow::Result<Option<Config>> {
    match args.command {
        Some(Command::Init) => {
            init_base_dir()?;
            tracing::info!("Configuration directory initialized in '~/.{}'. Make sure to fill out the 'config.json' file.", env!("CARGO_PKG_NAME"));
            Ok(None)
        }
        Some(Command::Config { source }) => {
            let config = match source {
                ConfigSource::Env => Config::from_env(),
                ConfigSource::Json { path } => Config::from_json(&path)?,
            };
            Ok(Some(config))
        }
        Some(Command::Exec { command }) => {
            let config = choose_config()?;
            command.handle_exec_command(config).await?;
            tracing::info!("Administrative command executed");
            Ok(None)
        }
        None => {
            let config = choose_config()?;
            Ok(Some(config))
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args = Args::parse();

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

    let config = match parse_cli(args).await {
        Ok(res) => match res {
            Some(c) => c,
            None => std::process::exit(0),
        },
        Err(e) => {
            tracing::error!("{}", e);
            panic!("{}", e);
        }
    };

    tracing::info!(
        config = config.redacted().as_value(),
        "Configuration loaded"
    );

    if let Err(e) = flatline::run(config).await {
        tracing::error!("{}", e);
        panic!("{}", e);
    }
}
