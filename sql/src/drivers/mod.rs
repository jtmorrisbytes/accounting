use std::sync::Arc;

pub mod migrations;
#[cfg(feature = "mysql")]
pub(super) mod mysql;
#[cfg(feature = "postgresql")]
pub(super) mod postgres;
#[cfg(feature = "sqlite")]
pub(super) mod sqlite;
// the driver contracts 'traits' to make it work with any driver
// any specific behavior should be defined here and then implemented in these subcrates

/*
    example:
    Trait Obj:
        performs sql functions
        A() -> T
        B() -> Y
        C() -> Z
    then each function gets implemented in the subcrates in this module for each specific sql Exectutor
    ex:
    impl Obj for PgPool {
        ....
    }
    which allows

    fn run_query(executor: [any executor that implements Obj]) -> {
        //cast executor as contract trait Obj then call its method
        (t as Obj).A()
    }
    // can also store as registry types
    DashMap<id,Arc<dyn Obj>>

    make sure you require Send + Sync in the trait definitions
    otherwise the compiler will complain that your trait object cannot be sent accross threads safely
*/

#[async_trait::async_trait]
/// the user implementation in this database crate is
/// is implemented via multiple databases. all user interactions go through
/// this trait.
pub trait UserTenantStore: Send + Sync {
    // async fn save_passkey(&self){}
}

/// Aquires a connection. Mysql, and postgres are single connections with this method, sqlite connections are POOLED, as SqliteConnection is not Send + Sync
pub async fn connect_tenant_any_url<S: AsRef<str>>(
    database_url: S,
    migrate: bool,
) -> Result<Arc<dyn UserTenantStore>, anyhow::Error> {
    sqlx::any::install_default_drivers();

    let url = database_url.as_ref();
    #[cfg(feature = "sqlite")]
    {
        // NOTE: Sqlite connections are NOT Send + Sync, so POOLING IS REQUIRED
        if url.starts_with("sqlite:///") {
            let c = sqlite::connect_pool_url(url).await?;
            if migrate {
                // use sqlx::{Connection};
                use sqlx::Acquire;

                let mut conn = c.acquire().await?;
                let mut tx = conn.begin().await?;
                self::migrations::user_multitenant::bring_up::<
                    barrel::backend::Sqlite,
                    sqlx::Sqlite,
                >(&mut tx)
                .await?;
                tx.commit().await?;
                // conn.close().await.ok();
            }
            let c: Arc<sqlx::Pool<sqlx::Sqlite>> = Arc::new(c);
            return Ok(c as Arc<dyn UserTenantStore>);
        }
    }
    #[cfg(feature = "postgresql")]
    {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            let mut c = postgres::connect_url(url).await?;
            if migrate {
                use sqlx::Connection;
                let mut tx = c.begin().await?;
                self::migrations::user_multitenant::bring_up::<barrel::backend::Pg, _>(&mut tx)
                    .await?;
                tx.commit().await?;
            }
            let c = Arc::new(c);
            return Ok(c as Arc<dyn UserTenantStore>);
        }
    }
    #[cfg(feature = "mysql")]
    {
        if url.starts_with("mysql://") {
            let mut c = mysql::connect_url(url).await?;
            if migrate {
                use sqlx::Acquire;
                let mut tx = c.begin().await?;
                let _ = self::migrations::user_multitenant::bring_up::<barrel::backend::MySql, _>(
                    &mut tx,
                )
                .await?;
                tx.commit().await?;
            }
            return mysql::connect_url(url)
                .await
                .map(|c| Arc::new(c) as Arc<dyn UserTenantStore>);
        }
    }

    return Err(anyhow::Error::msg(
        "Database crate compiled without any drivers available or invalid connection url. unable to make a connection",
    ));
}

/// Aquires a Pool with default options. Mysql, and postgres are POOLED connections with this method, sqlite connections are POOLED, as SqliteConnection is not Send + Sync
pub async fn connect_tenant_any_pool_url<S: AsRef<str>>(
    database_url: S,
    migrate: bool,
) -> Result<Arc<dyn UserTenantStore>, anyhow::Error> {
    sqlx::any::install_default_drivers();
    use sqlx::Acquire;

    let url = database_url.as_ref();
    #[cfg(feature = "sqlite")]
    {
        if url.starts_with("sqlite:///") {
            let p = sqlite::connect_pool_url(url).await?;
            if migrate {
                let mut c = p.acquire().await?;
                let mut tx = c.begin().await?;
                self::migrations::user_multitenant::bring_up::<barrel::backend::Sqlite, _>(&mut tx)
                    .await?;
            }
            return Ok(Arc::new(p) as Arc<dyn UserTenantStore>);
        }
    }
    #[cfg(feature = "postgresql")]
    {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            let p = postgres::connect_pool_url(url).await?;
            let mut c = p.acquire().await?;
            let mut tx = c.begin().await?;
            self::migrations::user_multitenant::bring_up::<barrel::backend::Sqlite, _>(&mut tx)
                .await?;
            return Ok(Arc::new(p) as Arc<dyn UserTenantStore>);
        }
    }
    #[cfg(feature = "mysql")]
    {
        if url.starts_with("mysql://") {
            // use sqlx::postgres::any::AnyConnectionBackend;

            let p = mysql::connect_pool_url(url).await?;
            let mut c = p.acquire().await?;
            let mut tx = c.begin().await?;
            self::migrations::user_multitenant::bring_up::<barrel::backend::MySql, _>(&mut tx)
                .await?;
            return Ok(Arc::new(p) as Arc<dyn UserTenantStore>);
        }
    }

    return Err(anyhow::Error::msg(
        "Database crate compiled without any drivers available or invalid connection url. unable to make a connection",
    ));
}

pub async fn run_user_multitenent_migrations_sqlite_pooled(
    p: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<(), anyhow::Error> {
    use sqlx::Acquire;
    // let p = sqlite::connect_pool_url(url).await?;
    let mut c = p.acquire().await?;
    let mut tx = c.begin().await?;
    self::migrations::user_multitenant::bring_up::<barrel::backend::Sqlite, _>(&mut tx).await?;
    return Ok(());
}

pub async fn run_user_multitenent_migrations_postgres_pooled(
    p: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), anyhow::Error> {
    use sqlx::Acquire;
    // let p = sqlite::connect_pool_url(url).await?;
    let mut c = p.acquire().await?;
    let mut tx = c.begin().await?;
    self::migrations::user_multitenant::bring_up::<barrel::backend::MySql, _>(&mut tx).await?;
    return Ok(());
}

pub async fn run_user_multitenant_migrations_mysql_pooled(
    p: &sqlx::Pool<sqlx::MySql>,
) -> Result<(), anyhow::Error> {
    use sqlx::Acquire;
    // let p = sqlite::connect_pool_url(url).await?;
    let mut c = p.acquire().await?;
    let mut tx = c.begin().await?;
    self::migrations::user_multitenant::bring_up::<barrel::backend::MySql, _>(&mut tx).await?;
    return Ok(());
}
