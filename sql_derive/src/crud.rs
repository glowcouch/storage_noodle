use proc_macro2::TokenStream;
use quote::quote;

/// Implementation of [`crate::Create`].
pub fn create(item: syn::ItemStruct) -> TokenStream {
    // Get the sql attribute(s). Error if there are none.
    let sql_attrs = match crate::attr::SqlAttr::from_item(item.clone()) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // Run `create_impl` on the attributes and return the resulting impl block(s).
    sql_attrs
        .iter()
        .map(|crate::attr::SqlAttr { backing_db, raw_id }| {
            create_impl(item.clone(), backing_db.clone(), raw_id.clone())
        })
        .collect()
}

/// Single-attribute implementation for [`create`].
fn create_impl(item: syn::ItemStruct, backing_db: syn::Type, raw_id: syn::Type) -> TokenStream {
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
                .bind(&self.#ident)
            }
        })
        .collect();

    // The SQL query to run.
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
        impl ::storage_noodle_sql::macro_helpers::Create<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident
        {
            type Error = ::sqlx::Error;

            fn create<'a>(
                &'a self,
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>
                + core::marker::Send
                + 'a,
            ) -> impl Future<Output = Result<::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>, Self::Error>> + Send {
                async move {
                    // Run the query, getting back a single value (the new row's id).
                    let query = ::sqlx::query_scalar(#query)#bind_calls;

                    // Get the raw id back from the query.
                    let raw = query.fetch_one(&storage.pool).await?;

                    Ok(::storage_noodle_sql::macro_helpers::AssocId::new(raw))
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
