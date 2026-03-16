use std::sync::Arc;

pub mod user_multitenant;
pub mod v1_initdb;


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
pub async fn bring_up<'exec,Exec,BarrelBackend,SqlxDatabase>(executor: Exec ) -> Result<(),anyhow::Error>
    where SqlxDatabase: sqlx::Database,
    Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
    BarrelBackend: barrel::backend::SqlGenerator {
    Ok(())
}
