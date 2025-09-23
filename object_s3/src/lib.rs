//! Provides S3 support to `storage_noodle_object`.

use minio::s3::types::S3Api;
use storage_noodle_object::Object;
use storage_noodle_traits::{Create, Delete, Read, Update};

/// S3 backing storage - only supports single-bucket usage.
pub struct S3Backing {
    /// The inner s3 client.
    pub client: minio::s3::Client,

    /// The bucket to use.
    pub bucket: String,
}

impl storage_noodle_traits::BackingStorage for S3Backing {
    type RawId = String;
}

/// Generates a random ID.
fn make_id() -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE};

    let random: [u8; 32] = rand::random();
    // 256 bit random number.

    // Use url-safe characters to encode the
    // random number.
    URL_SAFE.encode(random)
}

impl Create<S3Backing> for Object {
    type Error = minio::s3::error::Error;

    async fn create<'a>(
        &'a self,
        storage: impl core::ops::Deref<Target = S3Backing> + 'a + Send,
    ) -> Result<
        storage_noodle_traits::AssocId<
            Self,
            <S3Backing as storage_noodle_traits::BackingStorage>::RawId,
        >,
        Self::Error,
    > {
        // Generate ID.
        let name = make_id();

        // Upload data.
        let result = storage
            .client
            .put_object(&storage.bucket, name, self.data.clone().into())
            .send()
            .await;

        //= traits/spec.md#create-trait
        //# * In the case of a failure, the future MUST return `Err()`.
        //= traits/spec.md#create-trait
        //# * In the case of a success, the future MUST return `Ok(AssocId<Self, RawId>)` - where the `AssocId` holds the Id of the newly created item.
        result.map(|response| storage_noodle_traits::AssocId::new(response.object))
    }
}

impl Read<S3Backing> for Object {
    type Error = minio::s3::error::Error;

    async fn read(
        storage: impl core::ops::Deref<Target = S3Backing> + Send,
        id: impl core::ops::Deref<
            Target = storage_noodle_traits::AssocId<
                Self,
                <S3Backing as storage_noodle_traits::BackingStorage>::RawId,
            >,
        > + Send,
    ) -> Result<Option<Self>, Self::Error> {
        // Get data.
        let result = storage
            .client
            .get_object(&storage.bucket, id.as_raw())
            .send()
            .await;

        // Return Ok(None) if the object doesn't exist.
        match result {
            Ok(response) => {
                // FIXME: using `to_bytes` is not optimal.
                let segmented_bytes = response.content.to_segmented_bytes().await?;
                let bytes = segmented_bytes.to_bytes();

                //= traits/spec.md#read-trait
                //# * In the case of a full success, the future MUST return `Ok(Some(Self))` - where `Self` is the result of the read.
                Ok(Some(Self { data: bytes }))
            }
            Err(e) => {
                if let minio::s3::error::Error::S3Error(s3e) = &e
                    && let minio::s3::error::ErrorCode::NoSuchKey = s3e.code
                {
                    //= traits/spec.md#read-trait
                    //# * In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
                    Ok(None)
                } else {
                    //= traits/spec.md#read-trait
                    //# * In the case of a failure, the future MUST return `Err()`.
                    Err(e)
                }
            }
        }
    }
}

impl Update<S3Backing> for Object {
    type Error = minio::s3::error::Error;

    async fn update<'a>(
        &'a self,
        storage: impl core::ops::Deref<Target = S3Backing> + 'a + Send,
        id: impl core::ops::Deref<
            Target = storage_noodle_traits::AssocId<
                Self,
                <S3Backing as storage_noodle_traits::BackingStorage>::RawId,
            >,
        > + Send,
    ) -> Result<Option<()>, Self::Error> {
        // Check that the object does exist
        if let Err(e) = storage
            .client
            .get_object(&storage.bucket, id.deref().as_raw())
            .send()
            .await
            && let minio::s3::error::Error::S3Error(s3e) = &e
        {
            if let minio::s3::error::ErrorCode::NoSuchKey = s3e.code {
                //= traits/spec.md#update-trait
                //# * In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
                return Ok(None);
            }
            //= traits/spec.md#update-trait
            //# * In the case of a failure, the future MUST return `Err()`.
            return Err(e);
        }

        // Upload data.
        let result = storage
            .client
            .put_object(
                &storage.bucket,
                id.deref().as_raw(),
                self.data.clone().into(),
            )
            .send()
            .await;

        //= traits/spec.md#update-trait
        //# * In the case of a failure, the future MUST return `Err()`.
        //= traits/spec.md#update-trait
        //# * In the case of a full success, the future MUST return `Ok(Some(()))`.
        result.map(|_| Some(()))
    }
}

impl Delete<S3Backing> for Object {
    type Error = minio::s3::error::Error;

    async fn delete(
        storage: impl core::ops::Deref<Target = S3Backing> + Send,
        id: impl core::ops::Deref<
            Target = storage_noodle_traits::AssocId<
                Self,
                <S3Backing as storage_noodle_traits::BackingStorage>::RawId,
            >,
        > + Send,
    ) -> Result<Option<()>, Self::Error> {
        // Delete the object.
        let result = storage
            .client
            .delete_object(&storage.bucket, id.deref().as_raw())
            .send()
            .await;

        // Return Ok(None) if the object doesn't exist.
        match result {
            //= traits/spec.md#delete-trait
            //# * In the case of a full success, the future MUST return `Ok(Some(()))`.
            Ok(_) => Ok(Some(())),
            Err(e) => {
                if let minio::s3::error::Error::S3Error(s3e) = &e
                    && let minio::s3::error::ErrorCode::NoSuchKey = s3e.code
                {
                    //= traits/spec.md#delete-trait
                    //# * In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
                    Ok(None)
                } else {
                    //= traits/spec.md#delete-trait
                    //# * In the case of a failure, the future MUST return `Err()`.
                    Err(e)
                }
            }
        }
    }
}
