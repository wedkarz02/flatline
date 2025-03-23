use std::net::SocketAddr;

use crate::database::SupportedDatabases;

#[derive(Clone)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,

    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub database_pool: u32,
    pub database_variant: SupportedDatabases,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("api_host", &self.api_host)
            .field("api_port", &self.api_port)
            .field("database_host", &self.database_host)
            .field("database_port", &self.database_port)
            .field("database_user", &self.database_user)
            .field("database_password", &"***")
            .field("database_name", &self.database_name)
            .field("database_pool", &self.database_pool)
            .field("database_variant", &self.database_variant)
            .finish()
    }
}

impl Config {
    pub fn parse() -> Config {
        let api_host = std::env::var("API_HOST").expect("API_HOST should be set");
        let api_port = std::env::var("API_PORT")
            .expect("API_PORT should be set")
            .parse()
            .expect("API_PORT should be of type u16");

        let database_host = std::env::var("DATABASE_HOST").expect("DATABASE_HOST should be set");
        let database_port = std::env::var("DATABASE_PORT")
            .expect("DATABASE_PORT should be set")
            .parse()
            .expect("DATABASE_PORT should be of type u16");
        let database_user = std::env::var("DATABASE_USER").expect("DATABASE_USER should be set");
        let database_password =
            std::env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD should be set");
        let database_name = std::env::var("DATABASE_NAME").expect("DATABASE_NAME should be set");
        let database_pool = std::env::var("DATABASE_POOL")
            .expect("DATABASE_POOL should be set")
            .parse()
            .expect("DATABASE_POOL should be of type u32");

        let database_variant: SupportedDatabases = std::env::var("DATABASE_VARIANT")
            .expect("DATABASE_VARIANT should be set")
            .parse()
            .expect("database not supported");

        Config {
            api_host,
            api_port,
            database_host,
            database_port,
            database_user,
            database_password,
            database_name,
            database_pool,
            database_variant,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.api_host, self.api_port)
            .parse()
            .expect("{}:{} should be a viable socket address")
    }

    pub fn database_uri(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.database_variant
                .to_string(),
            self.database_user,
            self.database_password,
            self.database_host,
            self.database_port,
            self.database_name
        )
    }
}
