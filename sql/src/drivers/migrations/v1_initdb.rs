use crate::drivers::migrations::BackendName;


// performs the reconciliaton logic for this version of the datbase
pub async fn up<'exec,Exec,BarrelBackend,SqlxDatabase>(e:Exec) -> Result<(),anyhow::Error>
    where 
    SqlxDatabase: sqlx::Database,
    Exec: sqlx::Executor<'exec,Database = SqlxDatabase>,
          BarrelBackend: barrel::backend::SqlGenerator
{
    Ok(())
}