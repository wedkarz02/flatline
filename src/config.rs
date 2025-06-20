use std::net::SocketAddr;

use serde::Serialize;

use crate::database::DatabaseVariant;

#[derive(Clone, Debug, Serialize)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,

    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub database_pool: u32,
    pub database_variant: DatabaseVariant,
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

        let database_variant: DatabaseVariant = std::env::var("DATABASE_VARIANT")
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
            .unwrap_or_else(|_| {
                panic!(
                    "{}:{} should be a viable socket address",
                    self.api_host, self.api_port
                )
            })
    }

    pub fn database_uri(&self) -> String {
        match self.database_variant {
            DatabaseVariant::Postgres | DatabaseVariant::MySql => format!(
                "{}://{}:{}@{}:{}/{}",
                self.database_variant,
                self.database_user,
                self.database_password,
                self.database_host,
                self.database_port,
                self.database_name
            ),
            DatabaseVariant::Sqlite => {
                format!("{}://{}", self.database_variant, self.database_name)
            }
            DatabaseVariant::Mock => "in-memory".to_string(),
        }
    }
}
