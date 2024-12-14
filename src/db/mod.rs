pub mod postgres;
pub mod sqlite;
pub mod tests_postgres;
pub mod tests_sqlite;

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    ASC,
    DESC,
}

// Re-export the types you want available at the db module level
pub use self::postgres::PostgresDiaryDB;
pub use self::sqlite::SQLiteDiaryDB;
