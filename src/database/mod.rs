#[cfg(feature = "server")]
use sqlx::{PgPool, postgres::PgPoolOptions};
#[cfg(feature = "server")]
use std::{env, error::Error, time::Duration};

pub mod models;
pub mod operations;

#[cfg(feature = "server")]
pub mod redis;

#[cfg(feature = "server")]
pub struct Database {
    pub pool: PgPool,
}

#[cfg(feature = "server")]
impl Database {
    /// 从必填的 `DATABASE_URL` 创建连接池，并执行版本化迁移。
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database_url = env::var("DATABASE_URL")?;
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
