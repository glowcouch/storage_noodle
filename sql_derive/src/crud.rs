use crate::attr::for_each_attr;
use proc_macro2::TokenStream;
use quote::quote;

/// Implementation of [`crate::Create`].
pub fn create(item: &syn::ItemStruct) -> TokenStream {
    for_each_attr(item, create_impl)
}

/// Implementation of [`crate::Read`].
pub fn read(item: &syn::ItemStruct) -> TokenStream {
    for_each_attr(item, read_impl)
}

/// Implementation of [`crate::Update`].
pub fn update(item: &syn::ItemStruct) -> TokenStream {
    for_each_attr(item, update_impl)
}

/// Implementation of [`crate::Delete`].
pub fn delete(item: &syn::ItemStruct) -> TokenStream {
    for_each_attr(item, delete_impl)
}

/// Per-attribute implementation for [`create`].
fn create_impl(item: &syn::ItemStruct, backing_db: &syn::Type, raw_id: &syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item.clone();

    // Split generics.
    let (impl_generics, type_generics, where_clause) =
        match crate::attr::split_generics_with_raw_id_attr(item, raw_id) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

    // The table name.
    let table = ident.to_string();

    // List of columns.
    let columns = crate::sql::Column::from_fields(&fields);

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
                .map(|i| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", "), // List of "$1" - to be filled in with bind calls.
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // List of `.bind()` calls to run on the query.
    let bind_calls: TokenStream = columns
        .iter()
        .map(|column| {
            let field = &column.ident;
            quote! {.bind(&self.#field)}
        })
        .collect();

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::macro_helpers::Create<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident #type_generics #where_clause
        {
            type Error = ::sqlx::Error;

            fn create<'a>(
                &'a self,
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>
                + 'a,
            ) -> impl ::core::future::Future<Output = ::core::result::Result<::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>, Self::Error>>  {
                async move {
                    // Build the query.
                    let query = ::sqlx::query_scalar(#query)#bind_calls;

                    // Get the raw id back from the query.
                    //= traits/spec.md#create-trait
                    //# * In the case of a failure, the future MUST return `Err()`.
                    let raw = query.fetch_one(&storage.pool).await?;

                    //= traits/spec.md#create-trait
                    //# * In the case of a success, the future MUST return `Ok(AssocId<Self, RawId>)` - where the `AssocId` holds the Id of the newly created item.
                    Ok(::storage_noodle_sql::macro_helpers::AssocId::new(raw))
                }
            }
        }
    }
}

/// Per-attribute implementation for [`read`].
fn read_impl(item: &syn::ItemStruct, backing_db: &syn::Type, raw_id: &syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item.clone();

    // Split generics.
    let (impl_generics, type_generics, where_clause) =
        match crate::attr::split_generics_with_raw_id_attr(item, raw_id) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

    // The table name.
    let table = ident.to_string();

    // List of columns.
    let columns = crate::sql::Column::from_fields(&fields);

    // The SQL query to run.
    let query = {
        let query = format!(
            "
                SELECT {} FROM {}
                WHERE {}=$1;
            ",
            columns
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .join(", "), // List of column names (in order).
            table,
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::macro_helpers::Read<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident #type_generics #where_clause
        {
            type Error = ::sqlx::Error;

            fn read(
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>,
                id: impl ::core::ops::Deref<Target = ::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>>
            ) -> impl ::core::future::Future<Output = ::core::result::Result<core::option::Option<Self>, Self::Error>>  {
                async move {
                    // Build the query.
                    let query = ::sqlx::query_as(#query).bind(id.as_raw());

                    // Get the row back from the query.
                    let result = query.fetch_one(&storage.pool).await;

                    match result {
                        //= traits/spec.md#read-trait
                        //# * In the case of a full success, the future MUST return `Ok(Some(Self))` - where `Self` is the result of the read.
                        Ok(row) => Ok(Some(row)),

                        //= traits/spec.md#read-trait
                        //# * In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
                        Err(::sqlx::Error::RowNotFound) => Ok(None),

                        //= traits/spec.md#read-trait
                        //# * In the case of a failure, the future MUST return `Err()`.
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }
}

/// Per-attribute implementation for [`update`].
fn update_impl(item: &syn::ItemStruct, backing_db: &syn::Type, raw_id: &syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item.clone();

    // Split generics.
    let (impl_generics, type_generics, where_clause) =
        match crate::attr::split_generics_with_raw_id_attr(item, raw_id) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

    // The table name.
    let table = ident.to_string();

    // List of columns.
    let columns = crate::sql::Column::from_fields(&fields);

    // The SQL query to run.
    let query = {
        //= traits/spec.md#update-trait
        //# * If the item already exists, it MUST be overwritten.
        let query = format!(
            // Basically upsert, include the id in the fields.
            "
                INSERT INTO {} ({})
                VALUES ({})
                ON CONFLICT DO UPDATE
            ",
            table,
            columns
                .iter()
                .map(|c| &*c.name)
                // include the id
                .chain(core::iter::once(crate::sql::ID_FIELD_NAME))
                .collect::<Vec<_>>()
                .join(", "), // List of column names (in order).
            {
                #[allow(
                    clippy::range_plus_one,
                    reason = "adding one here because of the id key"
                )]
                // include the id
                (0..(columns.len() + 1))
                    .map(|i| format!("${}", i + 1))
                    .collect::<Vec<_>>()
                    .join(", ") // List of "$1" - to be filled in with bind calls.
            }
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // List of `.bind()` calls to run on the query.
    let bind_calls: TokenStream = columns
        .iter()
        .map(|column| {
            let field = &column.ident;
            quote! {.bind(&self.#field)}
        })
        .collect();

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::macro_helpers::Update<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident #type_generics #where_clause
        {
            type Error = ::sqlx::Error;

            fn update<'a>(
                &'a self,
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>
                + 'a,
                id: impl ::core::ops::Deref<Target = ::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>>
            ) -> impl ::core::future::Future<Output = ::core::result::Result<(), Self::Error>>  {
                async move {
                    // Build the query.
                    // Bind the id last, as this is where it was placed in the query.
                    let query = ::sqlx::query(#query)#bind_calls.bind(id.as_raw());

                    // Execute the query.
                    let result = query.execute(&storage.pool).await;

                    match result {
                        //= traits/spec.md#update-trait
                        //# * In the case of a success, the future MUST return `Ok(())`.
                        Ok(_) => Ok(()),

                        //= traits/spec.md#update-trait
                        //# * In the case of a failure, the future MUST return `Err()`.
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }
}

/// Per-attribute implementation for [`delete`].
fn delete_impl(item: &syn::ItemStruct, backing_db: &syn::Type, raw_id: &syn::Type) -> TokenStream {
    let syn::ItemStruct { ident, .. } = item.clone();

    // Split generics.
    let (impl_generics, type_generics, where_clause) =
        match crate::attr::split_generics_with_raw_id_attr(item, raw_id) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

    // The table name.
    let table = ident.to_string();

    // The SQL query to run.
    let query = {
        let query = format!(
            "
                DELETE FROM {}
                WHERE {}=$1;
            ",
            table,
            crate::sql::ID_FIELD_NAME,
        );
        syn::LitStr::new(&query, proc_macro2::Span::mixed_site())
    };

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::macro_helpers::Delete<::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>> for #ident #type_generics #where_clause
        {
            type Error = ::sqlx::Error;

            fn delete(
                storage: impl ::core::ops::Deref<Target = ::storage_noodle_sql::SqlBacking<#backing_db, #raw_id>>,
                id: impl ::core::ops::Deref<Target = ::storage_noodle_sql::macro_helpers::AssocId<Self, #raw_id>>
            ) -> impl Future<Output = ::core::result::Result<::core::option::Option<()>, Self::Error>>  {
                async move {
                    // Build & execute the query.
                    let result = ::sqlx::query(#query).bind(id.as_raw()).execute(&storage.pool).await;

                    match result {
                        //= traits/spec.md#delete-trait
                        //# * In the case of a full success, the future MUST return `Ok(Some(()))`.
                        Ok(row) => Ok(Some(())),

                        //= traits/spec.md#delete-trait
                        //# * In the case of a partial success, where the operation succeeded, but the item doesn't exist, the future MUST return `Ok(None)`.
                        Err(::sqlx::Error::RowNotFound) => Ok(None),

                        //= traits/spec.md#delete-trait
                        //# * In the case of a failure, the future MUST return `Err()`.
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }
}
