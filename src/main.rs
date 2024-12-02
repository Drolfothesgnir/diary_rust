use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};

#[derive(Debug, sqlx::FromRow)]
struct Entry {
    id: i64,
    content: String,
    created_at: NaiveDateTime,
    updated_at: Option<NaiveDateTime>,
    pinned: bool,
}

enum SortOrder {
    ASC,
    DESC,
}

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

    async fn read_entries(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
        sort: Option<SortOrder>,
        pinned: Option<bool>,
        substring: Option<&str>,
    ) -> Result<Vec<Entry>> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(10);
        let sort = sort.unwrap_or(SortOrder::DESC);

        let order = match sort {
            SortOrder::ASC => "ASC",
            SortOrder::DESC => "DESC",
        };

        let skip = (page - 1) * per_page;
        let mut conditions = Vec::new();
        if let Some(is_pinned) = pinned {
            conditions.push(format!("pinned = {}", is_pinned));
        }
        if let Some(substr) = substring {
            conditions.push(format!("content LIKE '%{}%'", substr));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let qry = format!(
            "
            SELECT * FROM entries
            {}
            ORDER BY created_at {}, id {}
            LIMIT $1 OFFSET $2;
        ",
            where_clause, order, order
        );

        sqlx::query_as::<_, Entry>(&qry)
            .bind(per_page)
            .bind(skip)
            .fetch_all(&self.pool)
            .await
            .context("Failed to read entries")
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
                created_at DATETIME NOT NULL DEFAULT (datetime('now')),
                updated_at DATETIME DEFAULT (datetime('now')),
                pinned     BOOLEAN NOT NULL DEFAULT 0
            );

            CREATE TRIGGER IF NOT EXISTS update_entries_updated_at
            AFTER UPDATE ON entries
            FOR EACH ROW
            BEGIN
                UPDATE entries
                SET updated_at = datetime('now')
                WHERE id = OLD.id;
            END;
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
        println!("\nDatabase connection closed\n")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = String::from("sqlite://sqlite.db");
    let db = SQLiteDiaryDB::new(&db_url).await.unwrap();

    let entries = db.read_entries(None, None, None, None, Some("ed")).await;

    print!("{:?}", entries);

    db.cleanup().await;

    Ok(())
}
