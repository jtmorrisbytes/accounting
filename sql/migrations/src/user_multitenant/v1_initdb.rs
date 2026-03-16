use barrel::{types,Migration};
use sqlx::Transaction;


pub async fn up<'exec,  BarrelBackend,SqlxDatabase>( tx:& mut Transaction<'_,SqlxDatabase>) -> Result<(),anyhow::Error>
    where 
    SqlxDatabase:sqlx::Database,
        for <'q> <SqlxDatabase as sqlx::database::Database>::Arguments<'q>: sqlx::IntoArguments<'q, SqlxDatabase>,
     for<'c> &'c mut <SqlxDatabase as sqlx::Database>::Connection: sqlx::Executor<'c, Database = SqlxDatabase>,
    // for<'e> &'e mut Exec: sqlx::Executor<'e,Database = SqlxDatabase>,
    BarrelBackend:barrel::backend::SqlGenerator

{

    Ok(())

}

pub  fn down() {
    // Error, cant go lower than this???
}

