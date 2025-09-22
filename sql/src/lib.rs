//! A backing storage implementation for [`storage_noodle_traits`].

use core::marker::PhantomData;

pub use storage_noodle_sql_derive::*;
use storage_noodle_traits::BackingStorage;

/// SQL schema generation functionality.
pub mod schema;

/// A SQL [`BackingStorage`] implementation.
#[derive(Debug, Clone)]
pub struct SqlBacking<DB: sqlx::Database, RawId> {
    /// The internal [`sqlx`] database pool.
    pub pool: sqlx::Pool<DB>,

    /// Phantom data.
    phantom: PhantomData<RawId>,
}

impl<DB: sqlx::Database, RawId> SqlBacking<DB, RawId> {
    /// Create a new instance.
    #[must_use]
    pub const fn new(pool: sqlx::Pool<DB>) -> Self {
        Self {
            pool,
            phantom: PhantomData,
        }
    }
}

impl<DB: sqlx::Database, RawId> BackingStorage for SqlBacking<DB, RawId> {
    type RawId = RawId;
}

#[doc(hidden)]
pub mod macro_helpers {
    pub use storage_noodle_traits::*;
}
