use core::marker::PhantomData;

pub use storage_noodle_sql_derive::*;
use storage_noodle_traits::BackingStorage;

/// Defines an SQL table for a type.
pub trait SqlTable {
    /// Should return a `CREATE TABLE` statement for the type.
    fn table_definition<'a, DB: sqlx::Database>() -> impl sqlx::Execute<'a, DB>;
}

pub trait SqlFlavor {
    type Key;
}

pub struct SqlBacking<F: SqlFlavor> {
    phantom: PhantomData<F>,
}

impl<F: SqlFlavor> BackingStorage for SqlBacking<F> {
    type RawId = F::Key;
}
