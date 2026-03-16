use std::sync::Arc;

use sqlx::Transaction;

use crate::reflect::SchemaInspector;

pub mod user_multitenant;
pub mod v1_initdb;
pub mod reflect;


pub enum BackendName {
    Postgresql,
    Mysql,
    Sqlite
}

// NOTE: advanced logic based on schema versions may not be supported
// you may have to call down and up yourself

// pub async fn migrate() {
//     // version?
// }
/// calls up for every module. 
/// uses reflection and migration code to make the database consistent. there may not be a "down" from this
pub async fn bring_up<'exec,BarrelBackend,SqlxDatabase>(tx: &mut Transaction<'_,SqlxDatabase> ) -> Result<(),anyhow::Error>
    where SqlxDatabase: sqlx::Database,
    

    for<'c> sqlx::Transaction<'c, SqlxDatabase>: SchemaInspector<SqlxDatabase>,


    for <'q> <SqlxDatabase as sqlx::database::Database>::Arguments<'q>: sqlx::IntoArguments<'q, SqlxDatabase>,
    for<'c> &'c mut <SqlxDatabase as sqlx::Database>::Connection: sqlx::Executor<'c, Database = SqlxDatabase>,
    // Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
    // Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
    BarrelBackend: barrel::backend::SqlGenerator {
    v1_initdb::up::<BarrelBackend,_>(&mut *tx).await?;
    Ok(())
}
