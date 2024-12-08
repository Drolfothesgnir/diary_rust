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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_display_without_updated_at() {
        let entry = Entry {
            id: 1,
            content: "Hello test".to_string(),
            created_at: NaiveDateTime::parse_from_str("2024-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: None,
            pinned: false,
        };

        let actual_output = entry.to_string();
        let expected_output = "Monday, January 1, 2024 2:30 PM\n-------------------------------------------\nHello test\n-------------------------------------------\n"
            .to_string();

        assert_eq!(actual_output, expected_output)
    }

    #[test]
    fn test_display_with_updated_at() {
        let entry = Entry {
            id: 1,
            content: "Hello test".to_string(),
            created_at: NaiveDateTime::parse_from_str("2024-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            updated_at: Some(
                NaiveDateTime::parse_from_str("2025-02-02 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
            pinned: false,
        };

        let actual_output = entry.to_string();
        let expected_output = "Monday, January 1, 2024 2:30 PM\n-------------------------------------------\nHello test\n-------------------------------------------\nUpdated at: Sunday, February 2, 2025 2:30 PM\n"
            .to_string();

        assert_eq!(actual_output, expected_output)
    }
}
