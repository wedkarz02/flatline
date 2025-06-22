use std::{fs, io, net::SocketAddr, path::PathBuf};

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::database::DatabaseVariant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,

    #[serde(deserialize_with = "serde_from_str::deserialize")]
    pub database_variant: DatabaseVariant,
    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub database_pool: u32,

    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    #[serde(with = "serde_minutes")]
    pub jwt_access_expiration: Duration,
    #[serde(with = "serde_days")]
    pub jwt_refresh_expiration: Duration,
}

impl Config {
    pub fn from_env() -> Config {
        let api_host = std::env::var("API_HOST").expect("API_HOST should be set");
        let api_port = std::env::var("API_PORT")
            .expect("API_PORT should be set")
            .parse()
            .expect("API_PORT should be of type u16");

        let database_variant: DatabaseVariant = std::env::var("DATABASE_VARIANT")
            .expect("DATABASE_VARIANT should be set")
            .parse()
            .expect("database not supported");
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

        let jwt_access_secret =
            std::env::var("JWT_ACCESS_SECRET").expect("JWT_ACCESS_SECRET should be set");
        let jwt_refresh_secret =
            std::env::var("JWT_REFRESH_SECRET").expect("JWT_REFRESH_SECRET should be set");
        let jwt_access_expiration_minutes = std::env::var("JWT_ACCESS_EXPIRATION")
            .expect("JWT_ACCESS_EXPIRATION should be set")
            .parse::<i64>()
            .expect("JWT_ACCESS_EXPIRATION should be of type i64");
        let jwt_access_expiration = chrono::Duration::minutes(jwt_access_expiration_minutes);
        let jwt_refresh_expiration_days = std::env::var("JWT_REFRESH_EXPIRATION")
            .expect("JWT_REFRESH_EXPIRATION should be set")
            .parse::<i64>()
            .expect("JWT_REFRESH_EXPIRATION should be of type i64");
        let jwt_refresh_expiration = chrono::Duration::days(jwt_refresh_expiration_days);

        Config {
            api_host,
            api_port,

            database_variant,
            database_host,
            database_port,
            database_user,
            database_password,
            database_name,
            database_pool,

            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_expiration,
            jwt_refresh_expiration,
        }
    }

    pub fn from_json(path: &PathBuf) -> anyhow::Result<Config> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
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

mod serde_minutes {
    use chrono::Duration;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(duration.num_minutes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let minutes = i64::deserialize(deserializer)?;
        Ok(Duration::minutes(minutes))
    }
}

mod serde_days {
    use chrono::Duration;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(duration.num_days())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let days = i64::deserialize(deserializer)?;
        Ok(Duration::days(days))
    }
}

mod serde_from_str {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(de::Error::custom)
    }
}
