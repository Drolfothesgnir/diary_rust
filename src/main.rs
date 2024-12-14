mod db;
mod models;

use anyhow::Result;
use db::PostgresDiaryDB;

#[tokio::main]
async fn main() -> Result<()> {
    let db_url = "postgresql://postgres:password@localhost:5432/diary_db";
    println!("Connecting to {}", db_url);

    let db = PostgresDiaryDB::new(db_url).await?;
    // 1. First, let's create an entry
    let new_entry = db.create_entry("My first diary entry!", false).await?;
    println!("Created entry: {}", new_entry.id);

    // 2. Read that specific entry
    let read_entry = db.read_entry(new_entry.id).await?;
    println!("Read entry: {}", read_entry);

    // 3. Let's create a few more entries for testing pagination
    db.create_entry("Second entry", true).await?; // pinned
    db.create_entry("Third entry", false).await?;
    db.create_entry("Fourth entry with special word", false)
        .await?;

    // 4. Test reading entries with different filters
    // - Get first page (default 10 per page)
    let entries = db.read_entries(None, None, None, None, None).await?;
    println!("All entries count: {}", entries.len());

    // - Get only pinned entries
    let pinned_entries = db.read_entries(None, None, None, Some(true), None).await?;
    println!("Pinned entries count: {}", pinned_entries.len());

    // - Search for entries containing "special"
    let search_entries = db
        .read_entries(None, None, None, None, Some("special"))
        .await?;
    println!("Entries with 'special': {}", search_entries.len());

    // 5. Update an entry
    let updated = db
        .update_entry(
            new_entry.id,
            Some("Updated content!".to_string()),
            Some(true),
        )
        .await?;
    println!("Updated entry: {}", updated);

    // 6. Delete an entry
    db.delete_entry(new_entry.id).await?;
    println!("Entry deleted!");

    // 7. Try to read the deleted entry (should error)
    match db.read_entry(new_entry.id).await {
        Ok(_) => println!("Entry still exists!"),
        Err(e) => println!("Expected error: {}", e),
    }

    db.close().await;

    Ok(())
}
