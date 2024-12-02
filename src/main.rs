use anyhow::{Context, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};

struct SQLiteDiaryDB {
    pool: SqlitePool,
}

impl SQLiteDiaryDB {
    async fn new(db_url: &str) -> Result<Self> {
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await.unwrap();
            match Self::create_schema(&db_url).await {
                Ok(pool) => {
                    println!("Database created successfully");
                    return Ok(Self { pool });
                }
                Err(e) => panic!("Error while creating database: {}", e),
            }
        }

        let pool = SqlitePool::connect(db_url)
            .await
            .context("Failed to connect to the database")
            .unwrap();

        Ok(Self { pool })
    }

    async fn create_entry(&self, content: &str, pinned: bool) -> Result<SqliteQueryResult> {
        let qry = "INSERT INTO entries (content, pinned) VALUES($1, $2);";
        let result = sqlx::query(qry)
            .bind(content)
            .bind(pinned)
            .execute(&self.pool)
            .await
            .context("Failed to create an entry");
        return result;
    }

    async fn create_schema(db_url: &str) -> Result<SqlitePool> {
        let pool = SqlitePool::connect(db_url)
            .await
            .context("Failed to connect to the database")
            .unwrap();

        let qry = "
            CREATE TABLE IF NOT EXISTS entries (
                id         INTEGER PRIMARY KEY NOT NULL,
                content    TEXT NOT NULL,
                created_at DATETIME DEFAULT (datetime('now', 'localtime')),
                updated_at DATETIME,
                pinned     BOOLEAN NOT NULL DEFAULT 0
            );
        ";

        sqlx::query(&qry)
            .execute(&pool)
            .await
            .context("Failed to create database schema")
            .unwrap();

        return Ok(pool);
    }

    async fn cleanup(&self) {
        self.pool.close().await;
        println!("Database connection closed")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = String::from("sqlite://sqlite.db");
    let db = SQLiteDiaryDB::new(&db_url).await.unwrap();
    match db.create_entry("Hello db", false).await {
        Ok(res) => println!("{:?}", res),
        Err(e) => panic!("{}", e),
    };
    db.cleanup().await;

    Ok(())
}
