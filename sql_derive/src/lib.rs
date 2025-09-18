//! Derives `storage_noodle` traits for an SQL backend.
//!
//! # Compile-time verification
//!
//! Currently we do not perform any compile-time verification of the generated SQL.
mod attr;
mod crud;
mod sql;

/// Derives `Create` for a type
#[proc_macro_derive(Create, attributes(config_noodle_sql))]
pub fn create(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::create(syn::parse_macro_input!(input)).into()
}

// Derives `Read` for a type
#[proc_macro_derive(Read)]
pub fn read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::read(syn::parse_macro_input!(input)).into()
}

// Derives `Update` for a type
#[proc_macro_derive(Update)]
pub fn update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::update(syn::parse_macro_input!(input)).into()
}

// Derives `Delete` for a type
#[proc_macro_derive(Delete)]
pub fn delete(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    crud::delete(syn::parse_macro_input!(input)).into()
}
