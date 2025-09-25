//! Integration test for S3 backing storage.
//!
//! Requires minio to be on the path.

use minio::s3::types::S3Api;
use storage_noodle_object::Object;
use storage_noodle_object_s3::S3Backing;
use storage_noodle_traits::{Create, Delete, Read, Update};

mod utils;

const BUCKET: &str = "integration-test";
const ADDR: &str = "http://127.0.0.1:9000";

#[tokio::test(flavor = "multi_thread")]
async fn main() {
    // Start minio.
    let _gaurd = utils::MinioGaurd::new(ADDR).await.unwrap();

    // Create client.
    let base_url: minio::s3::http::BaseUrl = ADDR.parse().unwrap();
    let static_provider = minio::s3::creds::StaticProvider::new("minioadmin", "minioadmin", None);
    let client =
        minio::s3::Client::new(base_url, Some(Box::new(static_provider)), None, None).unwrap();

    // Create bucket.
    if !client.bucket_exists(BUCKET).send().await.unwrap().exists {
        client.create_bucket(BUCKET).send().await.unwrap();
    }

    // Create backing storage.
    let backing = S3Backing {
        client,
        bucket: BUCKET.into(),
    };

    // Create an object.
    let object = Object {
        data: "Hello, World!".into(),
    };

    // Upload the object.
    let id = object.create(&backing).await.unwrap();

    // Read the object back, and assert that it has not changed.
    assert_eq!(Some(object), Object::read(&backing, &id).await.unwrap());

    // Create a new object
    let new_object = Object {
        data: "chocolate".into(),
    };

    // Update the object.
    new_object.update(&backing, &id).await.unwrap();

    // Read the object back, and assert that it has been sucessfully updated.
    assert_eq!(Some(new_object), Object::read(&backing, &id).await.unwrap());

    // Delete the object.
    Object::delete(&backing, &id).await.unwrap();

    // Assert that the object does not exist.
    assert_eq!(None, Object::read(&backing, &id).await.unwrap());
}
