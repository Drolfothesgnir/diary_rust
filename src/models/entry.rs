use chrono::NaiveDateTime;
use sqlx;
use std::fmt;

#[derive(Debug, sqlx::FromRow)]
pub struct Entry {
    pub id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub pinned: bool,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.created_at.format("%A, %B %-d, %Y %-I:%M %p"))?;
        writeln!(f, "-------------------------------------------")?;
        writeln!(f, "{}", self.content)?;
        writeln!(f, "-------------------------------------------")?;

        if let Some(date) = self.updated_at {
            writeln!(f, "Updated at: {}", date.format("%A, %B %-d, %Y %-I:%M %p"))?;
        }

        Ok(())
    }
}
