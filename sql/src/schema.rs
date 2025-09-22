#[cfg(feature = "sqlite_schema")]
pub mod sqlite;

/// Represents an SQL table.
#[derive(Debug)]
pub struct SqlTable {
    /// The table's name.
    pub name: String,

    /// The table's columns.
    pub columns: Vec<SqlColumn>,
}

/// Represents an SQL column.
#[derive(Debug)]
pub struct SqlColumn {
    /// The column's name.
    pub name: String,

    /// The column's type.
    pub ty: String,

    /// The column's `ColumnType`.
    pub column_type: ColumnType,
}

/// Represents what kind of column a column is.
#[derive(Debug)]
pub enum ColumnType {
    /// A data column.
    Data,

    /// A primary key column.
    PrimaryKey,
}

/// Trait for generating SQL schemas for a type.
pub trait MakeSqlTable<DB: sqlx::Database> {
    /// Returns the SQL table for the type.
    fn table() -> SqlTable;
}

/// Builder API for constructing full SQL schemas.
#[derive(Debug)]
pub struct SchemaBuilder<G: Fn(&SqlTable) -> String, DB: sqlx::Database> {
    /// The tables that have been added to the builder.
    tables: Vec<SqlTable>,

    /// The function that generates the SQL schema for a table.
    generator: G,

    /// Phantom data.
    phantom: core::marker::PhantomData<DB>,
}

impl<G: Fn(&SqlTable) -> String, DB: sqlx::Database> SchemaBuilder<G, DB> {
    /// Create a new instance.
    pub const fn new(builder: G) -> Self {
        Self {
            tables: Vec::new(),
            generator: builder,
            phantom: core::marker::PhantomData,
        }
    }

    /// Add a type to the schema.
    #[must_use]
    pub fn add_type<T: MakeSqlTable<DB>>(mut self) -> Self {
        self.tables.push(T::table());
        self
    }

    /// Build the schema.
    pub fn build(&self) -> String {
        self.tables.iter().map(|t| (self.generator)(t)).collect()
    }
}
