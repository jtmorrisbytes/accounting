
use sqlx::{Database, Executor, Row, prelude::FromRow};

pub const SQL_GET_ALL_TABLE_METADATA: &str = "SELECT name,* FROM sqlite_master WHERE type='table' and name NOT LIKE 'sqlite_%"; 

#[macro_export]
macro_rules! query_get_all_table_metadata {
    () => {
        sqlx::query($crate::reflect::sqlite::SQL_GET_ALL_TABLE_METADATA)
    };
}

pub const PRAGMA_TABLE_INFO_SQL: &str =
    r#"SELECT "cid","name","type","notnull","dflt_value","pk" from pragma_table_info(?)"#;


#[macro_export]
macro_rules! query_sqlite_get_all_column_metdata_for_table {
    ($table_name:ident) => {
        sqlx::query_as::<_,$crate::reflect::sqlite::SqliteColumnInfo,>($crate::reflect::sqlite::PRAGMA_TABLE_INFO_SQL).bind($table_name)
    }
}

#[derive(FromRow, Debug)]
pub struct SqliteColumnInfo {
    pub cid: i32,
    pub name: String,
    pub r#type: String,
    pub notnull: i32,
    pub dflt_value: Option<String>,
    pub pk: Option<i32>,
}



#[async_trait::async_trait]
impl super::SchemaInspector<sqlx::Sqlite> for sqlx::Transaction<'_, sqlx::Sqlite> {
    type ColumnInfo = SqliteColumnInfo;
    type TableInfo = ();
    async fn get_columns(
        &mut self,
        for_table_name: &str,
    ) -> Result<Vec<Self::ColumnInfo>, anyhow::Error> {
        let q = query_sqlite_get_all_column_metdata_for_table!(for_table_name);
        let t = q.fetch_all(&mut **self).await?;
        Ok(t)
    }
}