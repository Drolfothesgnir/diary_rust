use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};
use std::fmt;

#[derive(Debug, sqlx::FromRow)]
struct Entry {
    id: i64,
    content: String,
    created_at: NaiveDateTime,
    updated_at: Option<NaiveDateTime>,
    pinned: bool,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.created_at.format("%A, %B %-d, %Y %-I:%M %p"))?;
        writeln!(f, "-------------------------------------------")?;
        writeln!(f, "{}", self.content)?;
        writeln!(f, "-------------------------------------------")?;

        if let Some(date) = self.updated_at {
            writeln!(f, "{}", date.format("%A, %B %-d, %Y %-I:%M %p"))?;
        }

        Ok(())
    }
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

    async fn read_entry(&self, id: i64) -> Result<Entry> {
        let qry = "
            SELECT * FROM entries
            WHERE id = $1;
        ";

        sqlx::query_as::<_, Entry>(&qry)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context(format!("Failed to read entry with id: {}", id))
    }

    async fn update_entry(
        &self,
        id: i64,
        content: Option<String>,
        pinned: Option<bool>,
    ) -> Result<Entry> {
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if content.is_some() {
            param_count += 1;
            query_parts.push(format!("content = ${}", param_count));
        }

        if pinned.is_some() {
            param_count += 1;
            query_parts.push(format!("pinned = ${}", param_count));
        }

        if content.is_none() && pinned.is_none() {
            return Err(anyhow::anyhow!(
                "At least one field must be provided for update"
            ));
        }

        let qry = format!(
            "
            UPDATE entries
            SET {}
            WHERE id = $1
            RETURNING *;
        ",
            query_parts.join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Entry>(&qry).bind(id);

        if let Some(new_content) = content {
            query_builder = query_builder.bind(new_content);
        }

        if let Some(new_pinned) = pinned {
            query_builder = query_builder.bind(new_pinned);
        }

        query_builder
            .fetch_one(&self.pool)
            .await
            .context(format!("Failed to update an entry with id: {}", id))
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

    async fn close(&self) {
        self.pool.close().await;
        println!("\nDatabase connection closed\n")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = String::from("sqlite://sqlite.db");
    let db = SQLiteDiaryDB::new(&db_url).await.unwrap();

    let entry = db.read_entry(1).await.unwrap();
    println!("{}", entry);

    let updated_entry = db
        .update_entry(10, Some(String::from("Update for id 10")), Some(true))
        .await
        .unwrap();

    println!("{}", updated_entry);

    db.close().await;

    Ok(())
}
