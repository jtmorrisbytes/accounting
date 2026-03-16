use std::{marker::PhantomData, ops::Deref};

use sqlx::{SqlitePool, prelude::FromRow, sqlite::SqliteConnectOptions};
use uuid::Uuid;

pub mod user;
pub mod core;
pub mod drivers;
pub mod registry;


// macro_rules! entity {
//     ($Ty:T) => {
//         impl $crate::Entity for $Ty {

//         }
//     };
// }

macro_rules! sqlite_impl {
    ($T:ty,$Table_literal:literal) => {
        impl $T
        where
            $T: $crate::Table + $crate::GetId + $crate::PrimaryKey + Self::Sized,
        {
            async fn async_get_one<'a, E>(executor: E, id: &Id<$T>) -> $T {
                sqlx::query_as!($T, concat!("src/scripts/sqlite/", $table_literal,))
                    .fetch_one(executor)
                    .await
            }
        }
    };
}

// sql_proc_macro::sqlite_impl!(User);

pub async fn connect_sqlite<S: AsRef<str>>(
    database_url: Option<S>,
) -> Result<sqlx::SqlitePool, anyhow::Error> {
    // ensure the data directory exists, optionally report error later
    // let _ = std::fs::create_dir_all(&data_directory).ok();
    // let database_name= format!(",user_id.to_string());
    // let options = SqliteConnectOptions::new().filename(data_directory.join("accounting.db")).create_if_missing(true).journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = SqlitePool::connect(
        &database_url
            .map(|s| s.as_ref().to_string())
            .unwrap_or("sqlite://./accounting.db".to_string()),
    )
    .await?;
    Ok(pool)
}
