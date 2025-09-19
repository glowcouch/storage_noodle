use proc_macro2::TokenStream;
use quote::quote;

/// Runs `func` on each attribute and returns a list of impl blocks.
fn for_each_attr(
    item: syn::ItemStruct,
    func: impl Fn(syn::ItemStruct, syn::Type, syn::Type) -> TokenStream,
) -> TokenStream {
    // Get the sql attribute(s). Error if there are none.
    let sql_attrs = match crate::attr::SqlAttr::from_item(item.clone()) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // Run `create_impl` on the attributes and return the resulting impl block(s).
    sql_attrs
        .iter()
        .map(|crate::attr::SqlAttr { backing_db, raw_id }| {
            func(item.clone(), backing_db.clone(), raw_id.clone())
        })
        .collect()
}

/// Implementation of [`crate::Create`].
pub fn create(item: syn::ItemStruct) -> TokenStream {
    for_each_attr(item, create_impl)
}

/// Implementation of [`crate::Read`].
pub fn read(item: syn::ItemStruct) -> TokenStream {
    for_each_attr(item, read_impl)
}

/// Implementation of [`crate::Update`].
pub fn update(item: syn::ItemStruct) -> TokenStream {
    todo!()
}

/// Implementation of [`crate::Delete`].
pub fn delete(item: syn::ItemStruct) -> TokenStream {
    todo!()
}

/// Per-attribute implementation for [`create`].
fn create_impl(item: syn::ItemStruct, backing_db: syn::Type, raw_id: syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item;

    // The table name.
    let table = ident.to_string();

    // List of columns.
    let columns = crate::sql::Column::from_fields(fields.clone());

    // The SQL query to run.
    let query = {
        let query = format!(
            "
                INSERT INTO {} ({})
                VALUES ({})
                RETURNING {};
            ",
            table,
            columns
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .join(", "), // List of column names (in order).
            (0..columns.len())
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", "), // List of "?" - to be filled in with bind calls.
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // List of `.bind()` calls to run on the query.
    let bind_calls: TokenStream = fields
        .iter()
        .map(|ident| {
            quote! {.bind(&self.#ident)}
        })
        .collect();

    // Implement the trait.
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

/// Per-attribute implementation for [`read`].
fn read_impl(item: syn::ItemStruct, backing_db: syn::Type, raw_id: syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item;

    // The table name.
    let table = ident.to_string();

    // The SQL query to run.
    let query = {
        let query = format!(
            "
                SELECT * FROM {}
                WHERE {}=?;
            ",
            table,
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // Implement the trait.
    quote! {
        impl ::storage_noodle_sql::macro_helpers::Read<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident
        {
            type Error = ::sqlx::Error;

            fn read(
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>
                + core::marker::Send,
                id: impl ::core::ops::Deref<Target = ::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>> + core::marker::Send
            ) -> impl Future<Output = Result<Self, Self::Error>> + Send {
                async move {
                    let query = ::sqlx::query_as(#query).bind(id.as_raw());

                    // Get the raw id back from the query.
                    query.fetch_one(&storage.pool).await
                }
            }
        }
    }
}
