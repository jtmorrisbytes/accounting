use std::str::FromStr;

use anyhow::Context;
use sqlx::{ConnectOptions, SqliteConnection};
/// Creates a connection to a sqlite database specified by database_url.
/// defaults to create_if_missing = true
pub async fn connect_url<S: AsRef<str>>(database_url:S) -> Result<SqliteConnection,anyhow::Error> {
    let mut o = sqlx::sqlite::SqliteConnectOptions::from_str(database_url.as_ref()).with_context(||"while parsing sqlite connection options from database url")?;
    o = o.create_if_missing(true);
    let c = o.connect().await.with_context(||"While attempting to connect to a sqlite_database via database_url")?;
    Ok(c)
}
pub async fn connect_pool_url<S: AsRef<str>>(database_url:S) -> Result<sqlx::sqlite::SqlitePool,anyhow::Error> {
    let o = sqlx::sqlite::SqliteConnectOptions::from_str(database_url.as_ref()).with_context(||"while parsing sqlite connection options from database url")?;
    let p = sqlx::sqlite::SqlitePoolOptions::default().connect_with(o).await.with_context(||"while attempting a pooled connection to a sqlite database via a database url")?;
    Ok(p)
}

impl super::UserTenantStore for sqlx::sqlite::SqlitePool {

}
