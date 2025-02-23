use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub postgres_pool: u32,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("api_host", &self.api_host)
            .field("api_port", &self.api_port)
            .field("postgres_host", &self.postgres_host)
            .field("postgres_port", &self.postgres_port)
            .field("postgres_user", &self.postgres_user)
            .field("postgres_password", &"***")
            .field("postgres_db", &self.postgres_db)
            .field("postgres_pool", &self.postgres_pool)
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

        let postgres_host = std::env::var("POSTGRES_HOST").expect("POSTGRES_HOST should be set");
        let postgres_port = std::env::var("POSTGRES_PORT")
            .expect("POSTGRES_PORT should be set")
            .parse()
            .expect("POSTGRES_PORT should be of type u16");
        let postgres_user = std::env::var("POSTGRES_USER").expect("POSTGRES_USER should be set");
        let postgres_password =
            std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD should be set");
        let postgres_db = std::env::var("POSTGRES_DB").expect("POSTGRES_DB should be set");
        let postgres_pool = std::env::var("POSTGRES_POOL")
            .expect("POSTGRES_POOL should be set")
            .parse()
            .expect("POSTGRES_POOL should be of type u32");

        Config {
            api_host,
            api_port,
            postgres_host,
            postgres_port,
            postgres_user,
            postgres_password,
            postgres_db,
            postgres_pool,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.api_host, self.api_port)
            .parse()
            .expect("{}:{} should be a viable socket address")
    }

    pub fn postgres_uri(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }
}
