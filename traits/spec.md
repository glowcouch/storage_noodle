# Storage Noodle Traits

Specification for proper implementation of storage noodle traits.

## CRUD Traits

The CRUD traits are used for basic data manipulation.

### Create Trait

The `Create` trait is used to create a new item in the backing storage. The `Create::create` async function has these return values:

* In the case of a failure, the future MUST return `Err()`.
* In the case of a success, the future MUST return `Ok(AssocId<Self, RawId>)` - where the `AssocId` holds the Id of the newly created item.

### Read Trait

The Read trait is used to read an item from the backing storage. The `Read::read` async function has these return values:

* In the case of a failure, the future MUST return `Err()`.
* In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
* In the case of a full success, the future MUST return `Ok(Some(Self))` - where `Self` is the result of the read.

### Update Trait

The Update trait is used to update an item in the backing storage. The `Update::update` async function has these return values:

* In the case of a failure, the future MUST return `Err()`.
* In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
* In the case of a full success, the future MUST return `Ok(Some(()))`.

### Delete Trait

The Delete trait is used to delete an item from the backing storage. The `Delete::delete` async function has these return values:

* In the case of a failure, the future MUST return `Err()`.
* In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
* In the case of a full success, the future MUST return `Ok(Some(()))`.
