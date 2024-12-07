mod db;
mod models;

use anyhow::Result;
use db::SQLiteDiaryDB;

#[tokio::main]
async fn main() -> Result<()> {
    // Use SQLite
    let db = SQLiteDiaryDB::new("sqlite://sqlite.db").await?;

    // Or use Postgres
    // let db = PostgresDB::new("postgres://user:pass@localhost/diary").await?;

    let entries = db
        .read_entries(None, None, None, Some(true), Some("ell"))
        .await?;

    for entry in entries {
        println!("{}", entry);
    }

    db.close().await;
    Ok(())
}
