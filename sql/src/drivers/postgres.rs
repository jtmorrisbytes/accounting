use std::str::FromStr;

use anyhow::Context;
use sqlx::ConnectOptions;

pub async fn connect_url<S: AsRef<str>>(database_url:S) -> Result<sqlx::postgres::PgConnection,anyhow::Error> {
    let o = sqlx::postgres::PgConnectOptions::from_str(database_url.as_ref()).with_context(|| "while parsing sqlite connection options from database url")?;
    let c = o.connect().await.context("While attempting to connect to a sqlite_database via database_url")?;
    Ok(c)
}
pub async fn connect_pool_url<S: AsRef<str>>(database_url:S) -> Result<sqlx::postgres::PgPool,anyhow::Error> {
    let o = sqlx::postgres::PgConnectOptions::from_str(database_url.as_ref()).with_context(|| "while parsing sqlite connection options from database url")?;
    let p = sqlx::postgres::PgPoolOptions::default().connect_with(o).await.with_context(|| "while attempting a pooled connection to a sqlite database via a database url")?;
    Ok(p)
}

impl super::UserTenantStore for sqlx::postgres::PgConnection {
    
}

impl super::UserTenantStore for sqlx::postgres::PgPool {
    
}