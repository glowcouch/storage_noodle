use core::{marker::PhantomData, ops::Deref};

pub trait BackingStorage {
    type RawId;
}

pub struct AssocId<T: ?Sized, RawId> {
    inner: RawId,
    phantom: PhantomData<T>,
}

impl<T: ?Sized, RawId> AssocId<T, RawId> {
    pub fn new(raw: RawId) -> Self {
        Self {
            inner: raw,
            phantom: PhantomData,
        }
    }

    pub fn as_raw(&self) -> &RawId {
        &self.inner
    }
}

pub trait Create<S: BackingStorage> {
    type Error;

    fn create<'a>(
        &'a self,
        storage: impl Deref<Target = S> + core::marker::Sync + 'a + core::marker::Send,
    ) -> impl Future<Output = Result<AssocId<Self, S::RawId>, Self::Error>> + Send;
}

pub trait Read<S: BackingStorage>: Sized {
    type Error;

    fn read(
        storage: impl Deref<Target = S> + core::marker::Sync + core::marker::Send,
        id: impl Deref<Target = AssocId<Self, S::RawId>> + core::marker::Send,
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
