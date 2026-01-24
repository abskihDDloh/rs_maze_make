use std::{env, sync::Arc, time::Duration};

use dotenv::dotenv;
use sea_orm::{ConnectOptions, Database, DbConn, DbErr};

pub async fn establish_connection(max_connections: Option<u32>) -> Result<Arc<DbConn>, DbErr> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not found.");

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(max_connections.unwrap_or(1))
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    //  DB接続のためのコネクションを生成
    Database::connect(opt).await.map(Arc::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_establish_connection_with_default_connections() {
        // DATABASE_URL が存在する場合のテスト
        dotenv().ok();

        if env::var("DATABASE_URL").is_ok() {
            let result = establish_connection(None).await;
            assert!(
                result.is_ok(),
                "Failed to establish connection with default settings"
            );
        }
    }

    #[tokio::test]
    async fn test_establish_connection_with_custom_connections() {
        // カスタム接続数でのテスト
        dotenv().ok();

        if env::var("DATABASE_URL").is_ok() {
            let result = establish_connection(Some(5)).await;
            assert!(
                result.is_ok(),
                "Failed to establish connection with custom max_connections"
            );
        }
    }
}
