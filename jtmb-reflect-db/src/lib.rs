#![feature(associated_type_defaults)]

pub mod drivers;

use sqlx::{Database, Row, prelude::FromRow};





#[async_trait::async_trait]
pub trait SchemaInspector<DB: sqlx::Database> {
    type ColumnInfo: Send + Sync;
    type TableInfo: Send + Sync;
    type InformationSchema: Send + Sync = ();
    async fn get_columns(
        &mut self,
        for_table_name: &str,
    ) -> Result<Vec<Self::ColumnInfo>, anyhow::Error>;

    async fn get_tables(&mut self)-> Result<Vec<Self::TableInfo>,anyhow::Error> {
        todo!("Implement this function")
    }
    // async fn get_metadata(&mut self) -> Result<Self::InformationSchema,anyhow::Error>;
}
