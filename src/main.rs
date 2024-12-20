mod cli;
mod config;
mod db;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{create_entry, delete_entry, read_entry, update_entry, Args, Mode};
use config::{Config, DEFAULT_DB_URL};
use db::DiaryDB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("{:?}", args);

    let config = match Config::from_file(&args.config) {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("Failed to load config file: {}", e);
            eprintln!(
                "Make sure {} exists and has the correct format",
                args.config
            );
            println!("Default config is used");
            Config {
                db_url: DEFAULT_DB_URL.to_string(),
            }
        }
    };

    println!("{:?}", config);
    let diary_db = DiaryDB::new(&config.db_url).await?;

    match args.mode {
        Mode::Create => create_entry(&diary_db, args).await?,
        Mode::Read => read_entry(&diary_db, args).await?,
        Mode::Delete => delete_entry(&diary_db, args).await?,
        Mode::Update => update_entry(&diary_db, args).await?,
    }

    diary_db.db.close().await;
    Ok(())
}
