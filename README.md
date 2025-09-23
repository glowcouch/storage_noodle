# storage_noodle - abstracting over storage backends

Storage noodle aims to abstract over storage backends and provide a consistent interface for interacting with them. Any backend from a json file to an SQL database have the same interface. Additionally, it leverages the rust type system to provide a type-checked interface.

## Backend crate structure

Backend crates should provide derive macros that implement the [`Create`], [`Read`], [`Update`], and [`Delete`] traits.

## Available backends

|Backend|Crate|Description|
|---|---|---|
|SQL|`storage_noodle_sql`|Provides sqlx as a storage backend, as well as schema generation.|
|S3|storage_noodle_object_s3|Provides s3 as a storage backend.|

## Feature flags

|Flag|Description|
|---|---|
|`sqlx`|Implements sqlx traits for `AssocId`|
