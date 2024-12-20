use clap::{Parser, ValueEnum};

use crate::{
    db::{DiaryDB, SortOrder},
    models::Entry,
};
use anyhow::{Error, Result};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    #[value(name = "c")]
    Create,
    #[value(name = "r")]
    Read,
    #[value(name = "u")]
    Update,
    #[value(name = "d")]
    Delete,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // #[arg(short, long)]
    #[arg(value_enum)]
    pub mode: Mode,

    #[arg(short, long)]
    pub id: Option<i64>,

    #[arg(short = 't', long)]
    pub content: Option<String>,

    #[arg(short, long)]
    pub pinned: Option<bool>,

    #[arg(short, long, default_value_t = String::from("config.ini"))]
    pub config: String,

    #[arg(long)]
    pub per_page: Option<i64>,

    #[arg(long)]
    pub page: Option<i64>,

    #[arg(value_enum)]
    pub sort: Option<SortOrder>,

    #[arg(long)]
    pub substr: Option<String>,
}

fn print_entries(entries: Vec<Entry>) {
    println!("\nFound {} entries.\n", entries.len());
    let str = entries
        .into_iter()
        .map(|entry| entry.to_string())
        .collect::<Vec<String>>()
        .join("\n\n");
    println!("{}", str);
}

pub async fn create_entry(db: &DiaryDB, args: Args) -> Result<()> {
    if args.content.is_none() {
        return Err(Error::msg("Content must be provided for this operation"));
    }

    db.db
        .create_entry(args.content.unwrap(), args.pinned.unwrap_or(false))
        .await?;

    Ok(())
}

pub async fn read_entry(db: &DiaryDB, args: Args) -> Result<()> {
    if let Some(id) = args.id {
        let entry = db.db.read_entry(id).await?;
        println!("{}", entry);

        return Ok(());
    }

    let entries = db
        .db
        .read_entries(
            args.page,
            args.per_page,
            args.sort,
            args.pinned,
            args.substr,
        )
        .await?;
    print_entries(entries);

    Ok(())
}

pub async fn delete_entry(db: &DiaryDB, args: Args) -> Result<()> {
    if args.id.is_none() {
        return Err(Error::msg("Entry ID must be provided for this operation."));
    }

    db.db.delete_entry(args.id.unwrap()).await?;

    Ok(())
}

pub async fn update_entry(db: &DiaryDB, args: Args) -> Result<()> {
    if args.id.is_none() {
        return Err(Error::msg("Entry ID must be provided for this operation."));
    }

    db.db
        .update_entry(args.id.unwrap(), args.content, args.pinned)
        .await?;

    Ok(())
}
