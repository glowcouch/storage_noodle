//! Provides object storage functionality to `storage_noodle_traits`.

/// Represents an object.
#[derive(Debug, PartialEq)]
pub struct Object {
    /// The data in the object.
    pub data: bytes::Bytes,
}
