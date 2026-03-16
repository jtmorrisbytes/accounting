pub mod probe;

use std::{collections::HashMap, fmt::format, i64};

use sqlx::{Connection, Database, Execute, Executor, Row, Transaction, prelude::FromRow};

use crate::SchemaInspector;
/// Unlike other tools. this DOES NOT FILTER OUT INFORMATION SCHEMA
pub const SQL_GET_ALL_TABLE_METADATA: &str =
    r#"SELECT "name","type",* FROM sqlite_master WHERE type='table'"#;

#[derive(FromRow, Debug)]
pub struct PragmaTableInfo {
    pub cid: i32,
    pub name: String,
    pub r#type: String,
    pub notnull: i32,
    pub dflt_value: Option<String>,
    pub pk: Option<i32>,
}

#[macro_export]
macro_rules! query_get_all_table_metadata {
    () => {
        sqlx::query($crate::drivers::sqlite::SQL_GET_ALL_TABLE_METADATA)
    };
}

pub const PRAGMA_TABLE_INFO_SQL: &str = r#"SELECT "cid","name","type","notnull","dflt_value","pk" from pragma_table_info(?) ORDER BY "cid" ASC"#;


#[derive(Debug)]
pub struct ColumnMetatada {
    pub table_name: String,
    pub column_id: i32,
    pub name: String,
    pub r#type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_pk: bool,
}
impl ColumnMetatada {
    pub fn from_column_info(value: &PragmaTableInfo, tbl_name: &str) -> Self {
        Self {
            table_name: tbl_name.to_string(),
            column_id: value.cid,
            name: value.name.to_owned(),
            r#type: value.r#type.to_owned(),
            nullable: value.notnull == 0,
            default_value: value.dflt_value.to_owned(),
            is_pk: value.pk.unwrap_or(0) != 0,
        }
    }
}

// impl std::convert::From<SqliteColumnInfo> for ColumnMetatada {
//     fn from(value: SqliteColumnInfo) -> Self {
//         Self {
//             column_id: value.cid,
//             name: value.name,
//             r#type:value.r#type,
//             nullable: value.notnull == 0,
//             default_value:value.dflt_value,
//             is_pk: value.pk.unwrap_or(0) !=0
//         }
//     }
// }

#[derive(Debug)]
pub struct TableMetadata {
    name: String,
    // columns:HashMap<String,ColumnMetatada>
}

#[derive(sqlx::FromRow, Debug)]
pub struct SqliteMetadataRow {
    pub r#type: String,
    pub name: String,
    pub tbl_name: String,
    pub rootpage: i32,
    pub sql: Option<String>,
}

pub enum Logic {
    And,
    Or,
}
impl std::fmt::Display for Logic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}




// Checks db if table T exists with column  and datatype. 
/// Only valid for sqlx::Sqlite and sqlx::Any. 
/// Using any other database will result in wrong binds or syntax
/// 
/// To ask if the table exists at all leave column_name and type empty.
/// 
/// To ask if the table exists with this column and any type leave the type empty.
/// 
/// To ask if the table exists with this column and a specific type include the name of the type.
/// 
/// in any case if you want to include hidden columns, set include_hidden to true
/// 
/// this function is only capable of asking for the specific type that pragma_table_xinfo reports,
/// including but not limited to any custom types that dont match to an actual type
pub(crate) async fn meta_fn_get_columns_for_table_with_datatype<Executor, Database>(
    executor: Executor,
    table_name: String,
    column_name: String,
    r#type: String,
    include_hidden: bool,
    limit:i64,
    offset:i64,
) -> Result<Vec<PragmaTableInfo>, anyhow::Error>
where
    // database bounds
    Database: sqlx::Database,
    //executor bounds
    for<'executor> Executor: sqlx::Executor<'executor, Database = Database>,
    // arguments
    for<'arguments> <Database as sqlx::Database>::Arguments<'arguments>:
        sqlx::IntoArguments<'arguments, Database>,
    //  type bounds
    for<'encode> String: sqlx::Encode<'encode, Database> + sqlx::Type<Database>,
    // from row
    // for<'row> (i32,): sqlx::FromRow<'row, <Database as sqlx::Database>::Row>,
    // for<'row> i32: sqlx::FromRow<'row, <Database as sqlx::Database>::Row>,
    // encode
    // for<'integer> i32: sqlx::Encode<'integer, Database> + sqlx::Type<Database>,

    for<'integer> i64: sqlx::Encode<'integer, Database> + sqlx::Type<Database>,


    for<'row> PragmaTableInfo: sqlx::FromRow<'row,<Database as sqlx::Database>::Row>
{
    let column_name = {
        if column_name.len() == 0 {
            String::new()
        } else {
            format!("%{column_name}%")
        }
    };

    let type_pattern = {
        if r#type.len() == 0 {
            String::new()
        } else {
            format!("%{}%", r#type)
        }
    };

    sqlx::query_as(r#"SELECT cid, name,type,pk,hidden,"notnull",dflt_value from pragma_table_xinfo(?) where (name LIKE ? OR ? like '') AND (type LIKE ? OR ? = '') AND (hidden = 0 OR ?) LIMIT ? OFFSET ?"#)
    .bind(table_name)
    .bind(column_name.clone())
    .bind(column_name)
    .bind(r#type)
    .bind(type_pattern)
    .bind(include_hidden as i64)
    .bind(limit)
    .bind(offset)
    .fetch_all(executor)
    .await
    .map_err(|e|e.into())
}
/// Asks database if Table exists with Column and datatype. Only valid for sqlx::Sqlite using any other database will result in wrong binds or syntax
pub async fn sqlite_meta_fn_get_columns_for_table_with_datatype<Executor>(
    executor: Executor,
    table_name: String,
    column_name: String,
    r#type: String,
    include_hidden: bool,
    limit:i64,
    offset:i64,
) -> Result<Vec<PragmaTableInfo>, anyhow::Error>
where
    for<'executor> Executor: sqlx::SqliteExecutor<'executor, Database = sqlx::Sqlite>,
    // for<'row> (i32,): sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    // for<'row> i32: sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    for<'row> PragmaTableInfo: sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    
    // encode
    // for<'integer> i32: sqlx::Encode<'integer, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,

    for<'integer> i64: sqlx::Encode<'integer, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,

{
    meta_fn_get_columns_for_table_with_datatype::<Executor, sqlx::Sqlite>(
        executor,
        table_name,
        column_name,
        r#type,
        include_hidden,
        limit,
        offset
    )
    .await
    
}


#[derive(FromRow,Debug)]
pub struct SqliteMasterInfo {
    pub r#type: String,
    pub name: String,
    pub tbl_name: String,
    pub rootpage: i64,
    pub sql: Option<String>
}

async fn meta_fn_sqlite_master<Executor,Database>(executor: Executor,object_type:String,name:String,table_name: String,rootpage:i64) -> Result<Vec<SqliteMasterInfo>,anyhow::Error>
    where 
    // database bounds
    Database: sqlx::Database,
    //executor bounds
    for<'executor> Executor: sqlx::Executor<'executor, Database = Database>,
    // for<'executor> &'executor mut Executor::Connection: sqlx::Executor<'executor>,
    for<'c> &'c mut Database::Connection: sqlx::Executor<'c, Database = Database>,
    // for<'c> &'c sqlx::Pool<Database::Connection>: sqlx::Executor<'c, Database = Database>,


    // arguments
    for<'arguments> <Database as sqlx::Database>::Arguments<'arguments>:
        sqlx::IntoArguments<'arguments, Database>,
    //  type bounds
    for<'encode> String: sqlx::Encode<'encode, Database> + sqlx::Type<Database>,
    // encode
    for<'integer> i64: sqlx::Encode<'integer, Database> + sqlx::Type<Database>,
    // for<'row> (i32,): sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    // for<'row> i32: sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    
    // from row
    for<'row> SqliteMasterInfo: sqlx::FromRow<'row,<Database as sqlx::Database>::Row>



{
    sqlx::query_as(
        r#"SELECT "type", "name", "tbl_name", "rootpage", "sql" from sqlite_master WHERE type like ? AND name like ? AND tbl_name LIKE ? AND (rootpage = ? OR ? < 0)"#
    )
    .bind(object_type)
    .bind(name)
    .bind(table_name)
    .bind(rootpage)
    .bind(rootpage)
    .fetch_all(executor)
    .await
    .map_err(|e|e.into())
}

pub async fn sqlite_meta_fn_sqlite_master<Executor>(executor: Executor,object_type:String,name:String,table_name: String,rootpage:i64) -> Result<Vec<SqliteMasterInfo>,anyhow::Error>
    where 
    //executor bounds
    for<'executor> Executor: sqlx::Executor<'executor, Database = sqlx::Sqlite>,
    // arguments
    for<'arguments> <sqlx::Sqlite as sqlx::Database>::Arguments<'arguments>:
        sqlx::IntoArguments<'arguments, sqlx::Sqlite>,
    //  type bounds
    for<'encode> String: sqlx::Encode<'encode, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
    // encode
    for<'integer> i64: sqlx::Encode<'integer, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,

    // for<'row> (i32,): sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    // for<'row> i32: sqlx::FromRow<'row, <sqlx::Sqlite as sqlx::Database>::Row>,
    // for<'row> i32: sqlx::FromRow<'row, sqlx::sqlite::SqliteRow>,
    // for<'row> (i32,): sqlx::FromRow<'row, sqlx::sqlite::SqliteRow>,


    
    // from row
    for<'row> SqliteMasterInfo: sqlx::FromRow<'row,<sqlx::Sqlite as sqlx::Database>::Row>

{
    meta_fn_sqlite_master::<Executor,sqlx::Sqlite>(executor, object_type, name, table_name, rootpage).await
}

pub async fn txn_meta_fn_sqlite_master<Database>(transaction: &mut sqlx::Transaction<'_,Database>,object_type:String,name:String,table_name: String,rootpage:i64) -> Result<Vec<SqliteMasterInfo>,anyhow::Error>
    where 
    Database: sqlx::Database,
    //executor bounds
    for<'c> &'c mut <Database as sqlx::Database>::Connection: sqlx::Executor<'c, Database = Database>,
    // for<'executor> sqlx::Transaction<'executor,Database>: sqlx::Executor<'executor, Database = Database>,
    // for<'c> &'c mut Database::Connection: sqlx::Executor<'c, Database = Database>,
    // arguments
    for<'q> <Database as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, Database>,
    //  type bounds
    for<'encode> String: sqlx::Encode<'encode, Database> + sqlx::Type<Database>,
    // encode
    for<'integer> i64: sqlx::Encode<'integer, Database> + sqlx::Type<Database>,
    // from row
    for<'row> SqliteMasterInfo: sqlx::FromRow<'row,<Database as sqlx::Database>::Row>

{
    let c = **transaction;
    meta_fn_sqlite_master::<&mut <Database as sqlx::Database>::Connection, Database>(&mut **transaction, object_type, name, table_name, rootpage).await
}




#[async_trait::async_trait]
impl<'a> crate::SchemaInspector<sqlx::Sqlite> for sqlx::Transaction<'a, sqlx::Sqlite> {
    type ColumnInfo = PragmaTableInfo;
    type TableInfo = ();
    async fn get_columns(
        &mut self,
        for_table_name: &str,
    ) -> Result<Vec<Self::ColumnInfo>, anyhow::Error> {
        let t = sqlite_meta_fn_get_columns_for_table_with_datatype(&mut **self,for_table_name.to_string(),"".to_string(),"".to_string(),true,i64::MAX,0_i64).await?;
        // let t = q.fetch_all(&mut **self).await?;
        Ok(t)
    }
    async fn get_tables(&mut self) -> Result<Vec<Self::TableInfo>, anyhow::Error> {
        Ok(vec![])
    }
  
}

async fn q<'q,E,DB>(e:E)->Result<(),anyhow::Error>
    where
    DB: sqlx::Database,
    for<'e> E: Executor<'e,Database = DB>,
    for <'e> &'e mut <DB as sqlx::Database>::Connection: Executor<'e,Database = DB>,
    for<'n> <DB as sqlx::Database>::Arguments<'n>: sqlx::IntoArguments<'n, DB>,

{
    sqlx::query("select 1").execute(e).await.map(|r|()).map_err(|e|e.into())
}


async fn c() -> Result<sqlx::SqliteConnection,anyhow::Error> {
    sqlx::SqliteConnection::connect("::memory::").await.map_err(|e| e.into())
}

async fn t() {
    let mut c = c().await.unwrap();
    let mut t = c.begin().await.unwrap();
    meta_fn_sqlite_master(&mut *t, "".to_string(), "".to_string(), "".to_string(), 0);
}