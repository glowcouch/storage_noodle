//! # Semantics
//!
//! Table names are copied directly from the struct name.
//!
//! Column names are copied directly from the field name.
//!
//! The Id field is defined by [`ID_FIELD_NAME`].

/// The standard name of the id field.
pub const ID_FIELD_NAME: &str = "Id";

/// Describes a SQL column.
pub struct Column {
    /// The column name.
    pub name: String,

    /// The struct field corrosponding to the column.
    pub ident: syn::Ident,
}

impl Column {
    /// Creates a list of [`Column`]s from a [`syn::Fields`].
    pub fn from_fields(value: &syn::Fields) -> Vec<Self> {
        value
            .iter()
            .enumerate()
            .map(|(i, field)| {
                // Fall back to the index if the field has no name (unit structs).
                let ident = field.ident.clone().unwrap_or_else(|| {
                    syn::Ident::new(&i.to_string(), proc_macro2::Span::mixed_site())
                });

                Self {
                    name: ident.to_string(),
                    ident,
                }
            })
            .collect()
    }
}
