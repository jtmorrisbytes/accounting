use barrel::{types,Migration};
use sqlx::Transaction;

use crate::drivers::migrations::BackendName;

pub async fn up<'exec,  BarrelBackend,SqlxDatabase>( tx:& mut Transaction<'_,SqlxDatabase>) -> Result<(),anyhow::Error>
    where 
    SqlxDatabase:sqlx::Database,
    for <'q> <SqlxDatabase as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, SqlxDatabase>,
     for<'c> &'c mut <SqlxDatabase as sqlx::Database>::Connection: sqlx::Executor<'c, Database = SqlxDatabase>,
    // for<'e> &'e mut Exec: sqlx::Executor<'e,Database = SqlxDatabase>,
    BarrelBackend:barrel::backend::SqlGenerator

{
    // the master user table, for the app database
    let mut m = Migration::new();
    m.create_table_if_not_exists("users", |table| {
        table.add_column("id", types::binary());
    });
    
    let sql = m.make::<BarrelBackend>();
    let q = sqlx::query(&sql).execute(&mut **tx).await?;
    Ok(())

}

pub  fn down() {
    // Error, cant go lower than this???
}

