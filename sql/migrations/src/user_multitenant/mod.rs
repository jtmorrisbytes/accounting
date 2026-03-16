use sqlx::Transaction;

pub mod v1_initdb;


// pub async fn bring_up_pooled<'exec,Exec,BarrelBackend,SqlxDatabase>(mut executor: Exec) -> Result<(),anyhow::Error>
//     where SqlxDatabase: sqlx::Database,
//     for <'q> <SqlxDatabase as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, SqlxDatabase>,
//     // Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
//     for<'e> &'e mut Exec: sqlx::Executor<'e,Database = SqlxDatabase>,
//     BarrelBackend: barrel::backend::SqlGenerator {
//     v1_initdb::up::<Exec,BarrelBackend,SqlxDatabase>(&mut executor).await?;
//     Ok(())
// }

pub async fn bring_up<'exec,BarrelBackend,SqlxDatabase>(mut tx: &mut sqlx::Transaction<'_,SqlxDatabase>) -> Result<(),anyhow::Error>
    where SqlxDatabase: sqlx::Database,
        for <'q> <SqlxDatabase as sqlx::database::Database>::Arguments<'q>: sqlx::IntoArguments<'q, SqlxDatabase>,
    for<'c> &'c mut <SqlxDatabase as sqlx::Database>::Connection: sqlx::Executor<'c, Database = SqlxDatabase>,
    // Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
    // for<'e> &'e mut Exec: sqlx::Executor<'e,Database = SqlxDatabase>,
    BarrelBackend: barrel::backend::SqlGenerator {
    v1_initdb::up::<BarrelBackend,SqlxDatabase>(&mut tx).await?;
    Ok(())
}
