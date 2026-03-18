use sqlx::Database;

use super::sqlite::{PRAGMA_TABLE_INFO_SQL, PragmaTableInfo};

// #[macro_use]
use crate::meta_query_sqlite_pragma_table_info;

#[derive(Debug)]
pub enum AnyColumnInfo {
    Sqlite(PragmaTableInfo),
}

#[async_trait::async_trait]
impl<'t> crate::SchemaInspector<sqlx::any::Any> for sqlx::Transaction<'t, sqlx::any::Any> {
    type ColumnInfo = AnyColumnInfo;
    type TableInfo = ();
    async fn get_columns(
        &mut self,
        for_table_name: &str,
    ) -> Result<Vec<Self::ColumnInfo>, anyhow::Error> {
        match self.backend_name() {
            sqlx::sqlite::Sqlite::NAME => {
                let q = meta_query_sqlite_pragma_table_info!(for_table_name);
                let t = q.fetch_all(&mut **self).await?;
                let t = t.into_iter().map(|t| AnyColumnInfo::Sqlite(t)).collect();
                dbg!(&t);
                Ok(t)
            }
            _ => {
                todo!("not implemented yet")
            }
        }
    }
    async fn get_tables(&mut self) -> Result<Vec<Self::TableInfo>, anyhow::Error> {
        match self.backend_name() {
            sqlx::sqlite::Sqlite::NAME => {
                let meta_query_pragma_table_info = crate::query_get_all_table_metadata!();
                let t = meta_query_pragma_table_info.fetch_all(&mut **self).await?;
            }
            _ => {
                todo!("not implemented yet")
            }
        }
        Ok(vec![])
    }
}

/// Asks Database if Table exists with Column and Datatype. Only valid with the any driver for sqlite
pub async fn sqlite_meta_fn_does_table_exist_with_column_and_datatype<Executor>(
    executor: Executor,
    table_name: String,
    column_name: String,
    r#type: String,
    include_hidden: bool,
) -> Result<bool, anyhow::Error>
where
    for<'executor> Executor: sqlx::any::AnyExecutor<'executor, Database = sqlx::any::Any>,
    for<'row> (i32,): sqlx::FromRow<'row, <sqlx::any::Any as sqlx::Database>::Row>,
    for<'row> i32: sqlx::FromRow<'row, <sqlx::any::Any as sqlx::Database>::Row>,
{
    super::sqlite::meta_fn_get_columns_for_table_with_datatype::<Executor, sqlx::any::Any>(
        executor,
        table_name,
        column_name,
        r#type,
        include_hidden,
    )
    .await
    .map(|t| t != 0)
}
