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
                Ok(Some(Self { data: bytes }))
            }
            Err(e) => {
                if let minio::s3::error::Error::S3Error(s3e) = &e {
                    if let minio::s3::error::ErrorCode::NoSuchKey = s3e.code {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                } else {
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
                return Ok(None);
            }
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
            Ok(_) => Ok(Some(())),
            Err(e) => {
                if let minio::s3::error::Error::S3Error(s3e) = &e {
                    if let minio::s3::error::ErrorCode::NoSuchKey = s3e.code {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }
}
