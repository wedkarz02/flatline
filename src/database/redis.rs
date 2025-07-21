use std::sync::Arc;

use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct RedisCache {
    conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisCache {
    pub async fn new(redis_uri: String) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_uri)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn tokens(&self) -> TokenBlacklist {
        TokenBlacklist {
            conn: self.conn.clone(),
            prefix: "token:",
        }
    }
}

pub struct TokenBlacklist {
    conn: Arc<Mutex<MultiplexedConnection>>,
    prefix: &'static str,
}

impl TokenBlacklist {
    pub async fn blacklist(&self, jti: Uuid, exp: i64) -> redis::RedisResult<()> {
        let key = format!("{}{}", self.prefix, jti);
        let mut conn = self.conn.lock().await;
        redis::cmd("SET")
            .arg(&key)
            .arg("blacklisted")
            .arg("EX")
            .arg(exp)
            .query_async(&mut *conn)
            .await
    }

    pub async fn is_blacklisted(&self, jti: Uuid) -> redis::RedisResult<bool> {
        let key = format!("{}{}", self.prefix, jti);
        let mut conn = self.conn.lock().await;
        redis::cmd("EXISTS").arg(&key).query_async(&mut *conn).await
    }
}
