use proc_macro2::TokenStream;
use quote::quote;

/// Implementation of [`crate::Create`].
pub fn create(item: syn::ItemStruct) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item;

    // The table name.
    let table = ident.to_string();

    // List of field idents.
    let fields = fields.iter().enumerate().map(|(index, field)| {
        // Fall back to the index if the field has no name (unit structs).
        if let Some(ident) = &field.ident {
            ident.to_owned()
        } else {
            syn::Ident::new(&index.to_string(), proc_macro2::Span::mixed_site())
        }
    });

    // List of column names.
    let columns = fields
        .clone()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>();

    // List of "?" to be filled in at runtime.
    let values = (0..columns.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    // List of `.bind()` calls to run on the query.
    let bind_calls: TokenStream = fields
        .map(|ident| {
            quote! {
                .bind(self.#ident)
            }
        })
        .collect();

    let query = {
        let query = format!(
            "
                INSERT INTO {} ({})
                VALUES ({})
                RETURNING {};
            ",
            table,
            columns.join(", "),
            values,
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    quote! {
        impl<DB: ::sqlx::Dtabase, RawId> ::storage_noodle_traits::Create<::storage_noodle_sql::SqlBacking<RawId>> for #ident {
            type Error = DB::QueryResult;

            fn create(
                &self,
                storage: impl ::core::marker::Deref<Target = ::storage_noodle_sql::SqlBacking<RawId>>,
                id: impl ::core::marker::Deref<Target = AssocId<Self, RawId>>,
            ) -> impl Future<Output = Result<(), Self::Error>> + Send {
                async {
                    let query = ::sqlx::query(#query)#bind_calls;
                    query.execute(&storage.pool).await?
                }
            }
        }
    }
}

/// Implementation of [`crate::Read`].
pub fn read(item: syn::ItemStruct) -> TokenStream {
    todo!()
}

/// Implementation of [`crate::Update`].
pub fn update(item: syn::ItemStruct) -> TokenStream {
    todo!()
}

/// Implementation of [`crate::Delete`].
pub fn delete(item: syn::ItemStruct) -> TokenStream {
    todo!()
}
