use axum::extract::Extension;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, FromRow};
use std::str::FromStr;
use std::sync::Arc;

pub struct Db {
    pool: SqlitePool,
}
#[derive(sqlx::FromRow)]
pub struct SumCounts {
    sum: i64,
}

impl Db {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_options = SqliteConnectOptions::from_str("sqlite:db.sqlite")?
            .create_if_missing(true)
            .disable_statement_logging()
            .to_owned();

        let pool = SqlitePoolOptions::new().connect_with(db_options).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS conns (
                ip TEXT NOT NULL,
                count INTEGER NOT NULL
            );",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Conn {
    ip: String, // IpAddr not valid in sqlx
    count: i64, // i16 not valid sqlx
}

async fn get_nconns(Extension(db): Extension<Arc<Db>>) -> Json<i64> {
    let result = sqlx::query_as::<_, SumCounts>("SELECT SUM(count) as sum FROM conns;")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    Json(result.sum)
}

async fn add_conn(Extension(db): Extension<Arc<Db>>, Json(conn): Json<Conn>) -> StatusCode {
    sqlx::query("INSERT INTO conns (ip, count) VALUES (?1, ?2);")
        .bind(conn.ip.to_string())
        .bind(conn.count)
        .execute(&db.pool)
        .await
        .unwrap();
    StatusCode::CREATED
}

async fn conns(Extension(db): Extension<Arc<Db>>) -> Json<Vec<Conn>> {
    Json(
        sqlx::query_as::<_, Conn>("SELECT ip, count FROM conns")
            .fetch_all(&db.pool)
            .await
            .unwrap(),
    )
}

// https://docs.rs/sqlx/latest/sqlx/sqlite/type.SqlitePool.html
