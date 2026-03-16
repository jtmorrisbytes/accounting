use std::{collections::HashMap, path::PathBuf, str::FromStr};

use dashmap::DashMap;
use sqlx::{Acquire, Connection, SqlitePool, sqlite::SqliteConnectOptions};

use crate::core::{GetId, Id, Table};


const ERROR_NO_DRIVERS_AVAILABLE: &str = "this operation cannot be completed because the database was compiled without any drivers enabled.";

#[derive(PartialEq,Eq,Hash)]
// #[sql_proc_macro::sqlite_impl]
pub struct User {
    pub id: Id,
    // #[sqlx(skip)]
    // connection: UserDatabase
}

pub enum UserDatabasePool {
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::sqlite::SqlitePool),
    #[cfg(feature = "postgresql")]
    Postgres(sqlx::postgres::PgPool),
    #[cfg(feature = "mysql")]
    MySql(sqlx::mysql::MySqlPool)
}
impl UserDatabasePool {
    pub async fn connect<S: AsRef<str>>(database_url: S) -> Result<Self,anyhow::Error> {
        
        if database_url.as_ref().starts_with("postgresql://") {
            // assume postgres
            #[cfg(not(feature = "postgresql"))] {
                return Err(anyhow::Error::msg("The database library was not compiled with postgres support. postgres is not available"))
            }
            #[cfg(feature = "postgresql")] {
                let o = sqlx::postgres::PgConnectOptions::from_str(database_url.as_ref())?;
                let pool = sqlx::postgres::PgPool::connect_with(o).await?;
                Result::Ok(UserDatabasePool::Postgres(pool))
            }
            
        }
        else if database_url.as_ref().starts_with("sqlite:///") {
            // assume sqlite
            #[cfg(feature = "sqlite")] {
               let o = sqlx::sqlite::SqliteConnectOptions::from_str(database_url.as_ref())?;
                let pool = sqlx::sqlite::SqlitePool::connect_with(o).await?;
                Result::Ok(UserDatabasePool::Sqlite(pool))
            }
            // default path
            #[cfg(not(feature = "sqlite"))]
            {
                return Err(anyhow::Error::msg("Database was not compiled with sqlite support enabled. cannot make a connection for sqlite"));
            }
        }
        else if database_url.as_ref().starts_with("mysql://") {
            // assume mysql
            #[cfg(feature = "mysql")] {
                let o = sqlx::mysql::MySqlConnectOptions::from_str(database_url.as_ref())?;
                let pool = sqlx::mysql::MySqlPool::connect_with(o).await?;
                Result::Ok(UserDatabasePool::MySql(pool))   
            }
            #[cfg(not(feature="mysql"))] {
                return Err(anyhow::Error::msg("Database was not compiled with mysql support enabled. cannot make a connection for mysql"));
            }
        }
        else  {
            return Err(anyhow::Error::msg("Unsupported or unknown database protocol"))
        }
        
    }
    pub async fn aquire(&self) -> Result<UserDbConnection,anyhow::Error> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(p) => {
                Ok(UserDbConnection::Sqlite(p.acquire().await?))
            }
            #[cfg(feature = "postgresql")]
            Self::Postgres(p) => {
                Ok(UserDbConnection::Postgres(p.acquire().await?))
            }
            #[cfg(feature = "mysql")]
            Self::MySql(p) => {
                Ok(UserDbConnection::MySql(p.acquire().await?))
            }
            #[allow(unreachable_patterns)]
            _=> {
                return Err(anyhow::Error::msg(ERROR_NO_DRIVERS_AVAILABLE))
            }

        }
    }
}

pub enum UserDbConnection {
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::pool::PoolConnection<sqlx::sqlite::Sqlite>),
    #[cfg(feature = "postgresql")]
    Postgres(sqlx::pool::PoolConnection<sqlx::postgres::Postgres>),
    #[cfg(feature = "mysql")]
    MySql(sqlx::pool::PoolConnection<sqlx::mysql::MySql>)
}
pub struct UserDatabases {
    pools: DashMap<Id,UserDatabasePool>,
    // data_directory: PathBuf
    // todo postgres
}

// let database_name= format!("user_{}.db",user_id.to_string());
// let options = SqliteConnectOptions::new().filename(data_directory.join(database_name)).create_if_missing(true).journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
// let pool = SqlitePool::connect_with(options).await?;

impl UserDatabases {
    pub async fn new() -> Self {
        sqlx::any::install_default_drivers();
        Self{
            pools: DashMap::new(),
            // data_directory
        }
    }
    pub async fn connect<S: AsRef<str>>(&mut self, database_url: S, user_id: Id) -> Result<UserDbConnection, anyhow::Error> {
        // let options = sqlx::any::AnyConnectOptions::from_str(database_url.as_ref())?;
        if let Some(pool) = self.pools.get(&user_id) {
            let c = pool.aquire().await?;
            return Ok(c)
        }
        // try sqlite first

       let p = UserDatabasePool::connect(database_url).await?;
       let c = p.aquire().await?;
       self.pools.insert(user_id, p);
        Ok(c)
    }
    pub async fn run_migrations(&self,id: &Id) -> Result<(),anyhow::Error> {
        let p = self.pools.get(id).ok_or_else(|| anyhow::Error::msg("pool not connected. make sure you connect() first"))?;
    
        // let c = p.aquire().await?;
        match p.value() {
            #[cfg(feature = "sqlite")]
            UserDatabasePool::Sqlite(p) => {
                let migrator = sqlx::migrate!("./migrations/sqlite/user");
                migrator.run(p).await?;
                Ok(())
            }
            #[cfg(feature = "postgresql")]
            UserDatabasePool::Postgres(p) => {
                let migrator = sqlx::migrate!("./migrations/postgres/user");
                migrator.run(p).await?;
                Ok(())
            }
            #[cfg(feature = "mysql")]
            UserDatabasePool::MySql(p) => {
                let migrator = sqlx::migrate!("./migrations/postgres/user");
                migrator.run(p).await?;
                Ok(())
            }
            
        }
    }
    // pub fn executor(&self) -> &sqlx::SqlitePool {
    //     &self.pool
    // }
}


impl User {
    // pub fn get(connection: &UserDbConnection,id: Id<self>)
    pub fn sqlite_get<'r,E>(executor: E,id: Id) -> Result<Self, anyhow::Error> where E: sqlx::Executor<'r,Database = sqlx::Sqlite> {
        todo!()
    }
    // 
//     pub async fn create(connection: &UserDbConnection,id: Option<Id<Self>>) -> Result<Self,anyhow::Error>{
//         let id = id.unwrap_or_default();
//         match connection {
//             #[cfg(feature = "sqlite")] 
//             UserDbConnection::Sqlite(c)=> sqlx::query_file!("./src/scripts/sqlite/user/insert.sql",id),
//             #[cfg(feature = "postgresql")]
//             UserDbConnection::Postgres(c) => sqlx::query_file!("./src/scripts/postgresql/user/insert.sql",id),
//             #[cfg(feature="mysql")]
//             UserDbConnection::MySql(c) => sqlx::query_file!("./src/scripts/mysql/insert.sql",id),
//             _=> {
//                 return Err(anyhow::Error::msg(ERROR_NO_DRIVERS_AVAILABLE))
//             }
//         }
//         // let q = sqlx::query!(r#"insert into "users" (id) VALUES(?)"#,id);
//         todo!()
//     }
}


impl Table for User {
    const TABLE_NAME: &'static str = "users";
}


impl GetId for User {
    fn id(&self) -> &Id where Self:Sized {
        &self.id
    }
}


// #[tokio::test]
// pub async fn test_user_create()-> Result<(), anyhow::Error> {
//     let primary = crate::connect_sqlite(Some("sqlite:///Y:/accounting.db")).await?;
//     let u = User::create(&primary, None).await?;
//     UserSqliteDatabase::connect(user_id, sqlite_data_directory).await;
//     Ok(())
// }