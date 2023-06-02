use crate::error::Error;
use sqlx::mysql::{self, MySqlPool};
use std::env;

pub async fn create_mysql_pool() -> Result<MySqlPool, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let pool = MySqlPool::connect(&database_url)
        .await
        .map_err(Error::DatabaseError)?;
    Ok(pool)
}

pub async fn run_migrations(pool: &MySqlPool) -> Result<(), Error> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(Error::DatabaseError)?;
    Ok(())
}
