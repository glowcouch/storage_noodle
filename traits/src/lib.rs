use core::{marker::PhantomData, ops::Deref};

pub trait BackingStorage {
    type RawId;
}

pub struct AssocId<T: ?Sized, RawId> {
    inner: RawId,
    phantom: PhantomData<T>,
}

impl<T: ?Sized, RawId> Deref for AssocId<T, RawId> {
    type Target = RawId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait Create<S: BackingStorage> {
    type Error;

    fn create(
        &self,
        storage: impl Deref<Target = S>,
        id: impl Deref<Target = AssocId<Self, S::RawId>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait Read<S: BackingStorage>: Sized {
    type Error;

    fn read(
        storage: impl Deref<Target = S>,
        id: impl Deref<Target = AssocId<Self, S::RawId>>,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub trait Update<S: BackingStorage> {
    type Error;

    fn update(
        &self,
        storage: impl Deref<Target = S>,
        id: impl Deref<Target = AssocId<Self, S::RawId>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait Delete<S: BackingStorage> {
    type Error;

    fn update(
        storage: impl Deref<Target = S>,
        id: impl Deref<Target = AssocId<Self, S::RawId>>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
