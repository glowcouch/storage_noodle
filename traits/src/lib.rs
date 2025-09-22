#![doc = include_str!(concat!(env!("OUT_DIR"), "/README-rustdocified.md"))]

use core::{marker::PhantomData, ops::Deref};

#[cfg(feature = "sqlx")]
pub mod sqlx;

/// A type that can store persistant data.
pub trait BackingStorage {
    /// The id type that is used to identify specific items.
    type RawId;
}

/// An Id that references a specific type.
#[derive(Debug, PartialEq, Eq)]
pub struct AssocId<T: ?Sized, RawId> {
    /// The inner raw id.
    inner: RawId,

    /// Phantom data.
    phantom: PhantomData<T>,
}

impl<T: ?Sized, RawId> AssocId<T, RawId> {
    /// Create a new instance.
    pub const fn new(raw: RawId) -> Self {
        Self {
            inner: raw,
            phantom: PhantomData,
        }
    }

    /// Get a reference to the inner raw Id.
    pub const fn as_raw(&self) -> &RawId {
        &self.inner
    }
}

/// Trait that abstracts over creating data in a storage backend.
pub trait Create<S: BackingStorage> {
    /// The error type that can be returned from [`Create::create`].
    type Error;

    /// Creates a new item in the storage backend. Returns the Id of the new item.
    fn create<'a>(
        &'a self,
        storage: impl Deref<Target = S> + 'a + Send,
    ) -> impl Future<Output = Result<AssocId<Self, S::RawId>, Self::Error>> + Send;
}

/// Trait that abstracts over reading data from a storage backend.
pub trait Read<S: BackingStorage>: Sized {
    /// The error type that can be returned from [`Read::read`].
    type Error;

    /// Reads an item from the storage backend. Returns the item.
    fn read(
        storage: impl Deref<Target = S> + Send,
        id: impl Deref<Target = AssocId<Self, S::RawId>> + Send,
    ) -> impl Future<Output = Result<Option<Self>, Self::Error>> + Send;
}

/// Trait that abstracts over updating data in a storage backend.
pub trait Update<S: BackingStorage> {
    /// The error type that can be returned from [`Update::update`].
    type Error;

    /// Updates an item in the storage backend. Will return [`None`] if the item doesn't exist.
    fn update<'a>(
        &'a self,
        storage: impl Deref<Target = S> + 'a + Send,
        id: impl Deref<Target = AssocId<Self, S::RawId>> + Send,
    ) -> impl Future<Output = Result<Option<()>, Self::Error>> + Send;
}

/// Trait that abstracts over deleting data from a storage backend.
pub trait Delete<S: BackingStorage>: Sized {
    /// The error type that can be returned from [`Delete::delete`].
    type Error;

    /// Deletes an item from the storage backend. Will return [`None`] if the item doesn't exist.
    fn delete(
        storage: impl Deref<Target = S> + Send,
        id: impl Deref<Target = AssocId<Self, S::RawId>> + Send,
    ) -> impl Future<Output = Result<Option<()>, Self::Error>> + Send;
}
