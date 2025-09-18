use core::marker::PhantomData;

pub use storage_noodle_sql_derive::*;
use storage_noodle_traits::BackingStorage;

#[derive(Debug, Clone)]
pub struct SqlBacking<DB: sqlx::Database, RawId> {
    pub pool: sqlx::Pool<DB>,
    phantom: PhantomData<RawId>,
}

impl<DB: sqlx::Database, RawId> SqlBacking<DB, RawId> {
    pub fn new(pool: sqlx::Pool<DB>) -> Self {
        Self {
            pool,
            phantom: PhantomData,
        }
    }
}

impl<DB: sqlx::Database, RawId> BackingStorage for SqlBacking<DB, RawId> {
    type RawId = RawId;
}
