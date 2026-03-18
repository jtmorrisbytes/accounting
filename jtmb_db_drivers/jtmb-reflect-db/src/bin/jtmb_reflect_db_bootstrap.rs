use std::str::FromStr;

use jtmb_reflect_db::SchemaInspector;
use sqlx::{ConnectOptions, Connection};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sqlx::any::install_default_drivers();
    let mut args = std::env::args();
    let _ = args.next();
    let database_url_env = std::env::var("DATABASE_URL").ok();
    let database_url = args.next();
    let database_url = database_url
        .or(database_url_env)
        .ok_or("Expected database url")?;

    let any_opts = sqlx::any::AnyConnectOptions::from_str(&database_url)?;
    let mut c = any_opts.connect().await?;
    let mut tx = c.begin().await?;

    let tables = tx.get_metadata().await?;

    Ok(())
}
