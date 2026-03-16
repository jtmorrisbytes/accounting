use std::str::FromStr;

use sqlx::{ConnectOptions, Connection, Database};

#[tokio::main]
pub async fn main() -> Result<(),anyhow::Error> {
    sqlx::any::install_default_drivers();
    let mut args = std::env::args();
    let _ = args.next();
    let primary_url = args.next().unwrap();
    dbg!(&primary_url);
    // let secondary_url = args.next().unwrap();


    let options = sqlx::any::AnyConnectOptions::from_str(&primary_url)?;
    // bring up the user multi db
    let mut c = options.connect().await?;
    // sql_migrations::user_multitenant::bring_up(tx)
    let name = c.backend_name().to_string();
    let mut tx = c.begin().await?;
    match name.as_str() {
        sqlx::MySql::NAME => {
            sql_migrations::bring_up::<barrel::backend::MySql,_>(&mut tx).await?;  
        } 
        sqlx::Postgres::NAME => {
            sql_migrations::bring_up::<barrel::backend::Pg,_>(&mut tx).await?;
        }
        sqlx::Sqlite::NAME => {
            sql_migrations::bring_up::<barrel::backend::Sqlite,_>(&mut tx).await?;
        }
        _=> {
            return Err(anyhow::Error::msg("Unsupported any backend"))
        }
    }
    Ok(())
}