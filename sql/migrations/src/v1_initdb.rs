use crate::reflect::SchemaInspector;
use barrel::{Migration, types};
use sea_query::{Alias, Expr, PostgresQueryBuilder, SqliteQueryBuilder};
use sqlx::{Database, Executor, Transaction};
// performs the reconciliaton logic for this version of the datbase
pub async fn up<'exec, BarrelBackend, SqlxDatabase>(
    tx: &mut Transaction<'_, SqlxDatabase>,
) -> Result<(), anyhow::Error>
where
    SqlxDatabase: sqlx::Database,
    for<'c> sqlx::Transaction<'c, SqlxDatabase>: SchemaInspector<SqlxDatabase>,
    for<'q> <SqlxDatabase as sqlx::database::Database>::Arguments<'q>:
        sqlx::IntoArguments<'q, SqlxDatabase>,
    for<'c> &'c mut <SqlxDatabase as sqlx::Database>::Connection:
        sqlx::Executor<'c, Database = SqlxDatabase>,
    BarrelBackend: barrel::backend::SqlGenerator,
{
    // the master user table, for the app database
    let mut m = Migration::new();
    m.create_table_if_not_exists("users", |table| {
        table.add_column("id", types::binary().nullable(false).unique(true));
    });

    let sql = m.make::<BarrelBackend>();
    let q = sqlx::query(&sql).execute(&mut **tx).await?;

    let q = tx.get_columns("users").await?;
    //     match SqlxDatabase::NAME {
    //     sqlx::Sqlite::NAME => {
    //         // tx.fetch_all(q).await?;
    //         // let m = ;
    //     }
    //     sqlx::Postgres::NAME => {
    //         // i_q.to_string(PostgresQueryBuilder)
    //         todo!()
    //     }
    //     sqlx::MySql::NAME => {
    //         // i_q.to_string(sea_query::MysqlQueryBuilder)
    //         todo!()
    //     }
    //     _=> {panic!("{}",SqlxDatabase::NAME)}
    // };

    let m = Migration::new().change_table("users", |table| {});
    Ok(())
}
