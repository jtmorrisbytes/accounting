use std::sync::Arc;

use dashmap::DashMap;

pub struct UserStoreFactory {
    pools: DashMap<crate::core::models::UserId,Arc<dyn crate::drivers::UserTenantStore>>
}
impl UserStoreFactory {
    pub fn new()-> Self {
        Self {
            pools: DashMap::new()
        }
    }
    pub async fn connect_pooled_url<S: AsRef<str>>(&mut self, id: &crate::core::models::UserId, database_url:S,run_migrations_on_connect:bool) -> Result<Arc<dyn crate::drivers::UserTenantStore>, anyhow::Error> {
        if let Some(p) = self.pools.get(id) {
            return Ok(p.clone())
        }
        let p = crate::drivers::connect_tenant_any_pool_url(database_url.as_ref(),run_migrations_on_connect).await?;
        let c = p.clone();
        self.pools.insert(id.clone(), p);
        Ok(c)
    }
        pub async fn connect_url<S: AsRef<str>>(&mut self, id: &crate::core::models::UserId, database_url:S,run_migrations_on_connect:bool) -> Result<Arc<dyn crate::drivers::UserTenantStore>, anyhow::Error> {
        if let Some(p) = self.pools.get(id) {
            return Ok(p.clone())
        }
        let p = crate::drivers::connect_tenant_any_url(database_url.as_ref(),run_migrations_on_connect).await?;
        let c = p.clone();
        self.pools.insert(id.clone(), p);
        Ok(c)
    }
}1