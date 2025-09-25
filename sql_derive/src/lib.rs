//! Derives `storage_noodle` traits for an SQL backend.
//!
//! # Compile-time verification
//!
//! Currently we do not perform any compile-time verification of the generated SQL.

/// Attribute-related utils.
mod attr;

/// Derives for `Create`, `Read`, `Update`, and `Delete` traits.
mod crud;

/// Derive for `SqlTable`.
mod schema;

/// SQL-related utils.
mod sql;

/// Derives `SqlTable` for a type
#[proc_macro_derive(SqlTable, attributes(storage_noodle_raw_id))]
pub fn sql_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    schema::sql_table(&syn::parse_macro_input!(input)).into()
}

/// Derives `Create` for a type
#[proc_macro_derive(Create, attributes(storage_noodle_sql, storage_noodle_raw_id))]
pub fn create(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::create(&syn::parse_macro_input!(input)).into()
}

/// Derives `Read` for a type
#[proc_macro_derive(Read, attributes(storage_noodle_sql, storage_noodle_raw_id))]
pub fn read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::read(&syn::parse_macro_input!(input)).into()
}

/// Derives `Update` for a type
#[proc_macro_derive(Update, attributes(storage_noodle_sql, storage_noodle_raw_id))]
pub fn update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::update(&syn::parse_macro_input!(input)).into()
}

/// Derives `Delete` for a type
#[proc_macro_derive(Delete, attributes(storage_noodle_sql, storage_noodle_raw_id))]
pub fn delete(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::delete(&syn::parse_macro_input!(input)).into()
}
