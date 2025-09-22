#[derive(Debug)]
pub struct SqlRow {
    pub name: String,
    pub ty: String,
}

pub trait SqlTable<DB: sqlx::Database> {
    fn rows() -> Vec<SqlRow>;
}
