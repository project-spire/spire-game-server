use crate::core::config::config;
use deadpool_postgres::{Config, Client, Pool, PoolError, Runtime};
use tokio_postgres::NoTls;

pub struct Resource {
    db_pool: Pool
}

impl Resource {
    pub async fn new() -> Resource {
        let mut pool_config = Config::new();
        pool_config.host = Some(config().db_host.clone());
        pool_config.port = Some(config().db_port);
        pool_config.user = Some(config().db_user.clone());
        pool_config.password = Some(config().db_password.clone());
        pool_config.dbname = Some(config().db_name.clone());

        let db_pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        _ = db_pool.get().await.unwrap(); // Connection check

        Resource {
            db_pool
        }
    }

    pub async fn db_client(&self) -> Result<Client, PoolError> {
        self.db_pool.get().await
    }
}
