use sqlx::Database;

use super::sqlite::{PRAGMA_TABLE_INFO_SQL, SqliteColumnInfo};

// #[macro_use]
use crate::query_sqlite_get_all_column_metdata_for_table;

#[derive(Debug)]
pub enum AnyColumnInfo {
    Sqlite(SqliteColumnInfo),
}

#[async_trait::async_trait]
impl<'t> super::SchemaInspector<sqlx::any::Any> for sqlx::Transaction<'t, sqlx::any::Any> {
    type ColumnInfo = AnyColumnInfo;
    type TableInfo = ();
    async fn get_columns(
        &mut self,
        for_table_name: &str,
    ) -> Result<Vec<Self::ColumnInfo>, anyhow::Error> {
        match self.backend_name() {
            sqlx::sqlite::Sqlite::NAME => {
                let q = query_sqlite_get_all_column_metdata_for_table!(for_table_name);
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
}
