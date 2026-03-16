use std::str::FromStr;

use anyhow::Context;
use sqlx::ConnectOptions;

pub async fn connect_url<S: AsRef<str>>(database_url:S) -> Result<sqlx::mysql::MySqlConnection,anyhow::Error> {
    let o = sqlx::mysql::MySqlConnectOptions::from_str(database_url.as_ref()).with_context(|| "while parsing sqlite connection options from database url")?;
    let c = o.connect().await.context("While attempting to connect to a sqlite_database via database_url")?;
    Ok(c)
}
pub async fn connect_pool_url<S: AsRef<str>>(database_url:S) -> Result<sqlx::mysql::MySqlPool,anyhow::Error> {
    let o = sqlx::mysql::MySqlConnectOptions::from_str(database_url.as_ref()).with_context(|| "while parsing sqlite connection options from database url")?;
    let p = sqlx::mysql::MySqlPoolOptions::default().connect_with(o).await.with_context(|| "while attempting a pooled connection to a sqlite database via a database url")?;
    Ok(p)
}

impl super::UserTenantStore for sqlx::mysql::MySqlConnection {
    
}

impl super::UserTenantStore for sqlx::mysql::MySqlPool {
    
}