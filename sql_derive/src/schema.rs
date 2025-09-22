use crate::attr::for_each_attr;
use proc_macro2::TokenStream;
use quote::quote;

/// Implementation of [`crate::SqlTable`].
pub fn sql_table(item: syn::ItemStruct) -> TokenStream {
    for_each_attr(item, sql_table_impl)
}

/// Per-attribute implementation of [`sql_table`].
pub fn sql_table_impl(
    item: syn::ItemStruct,
    backing_db: syn::Type,
    raw_id: syn::Type,
) -> TokenStream {
    let syn::ItemStruct { ident, fields, .. } = item.clone();

    // Get the raw id generic from attributes (if it exists).
    let raw_id_generic = match crate::attr::raw_id_attr(item.clone()) {
        Some(result) => match result {
            Ok(v) => Some(v),
            Err(e) => return e.to_compile_error(),
        },
        None => None,
    };

    // Split generics.
    let (impl_generics, type_generics, where_clause) =
        match crate::attr::split_generics_with_raw_id_attr(item.clone(), raw_id.clone()) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

    // List of sql rows to put in the vec.
    let sql_rows = fields
        .iter()
        .enumerate()
        .map(|(index, syn::Field { ident, ty, .. })| {
            // Fall back to index if there is no field name (unit structs).
            let name = syn::LitStr::new(
                &ident
                    .clone()
                    .map(|id| id.to_string())
                    .unwrap_or(index.to_string()),
                proc_macro2::Span::call_site(),
            );

            // Replace the id generic (if it exists) with the raw id concrete type.
            let processed_ty = match &raw_id_generic {
                Some(raw_id_generic) => crate::attr::make_concrete(ty, raw_id_generic, &raw_id),
                None => ty.clone(),
            };

            quote! {
                ::storage_noodle_sql::schema::SqlColumn {
                    name: #name.to_string(),
                    ty: ::sqlx::TypeInfo::name(&<#processed_ty as ::sqlx::Type<#backing_db>>::type_info()).to_string(),
                    column_type: ::storage_noodle_sql::schema::ColumnType::Data,
                }
            }
        });

    let sql_rows_punctuated =
        itertools::Itertools::intersperse(sql_rows, quote! {,}).collect::<TokenStream>();

    // Extra id (primary key) column.
    let id_column = {
        let name = syn::LitStr::new(crate::sql::ID_FIELD_NAME, proc_macro2::Span::call_site());
        quote! {
            ::storage_noodle_sql::schema::SqlColumn {
                name: #name.to_string(),
                ty: ::sqlx::TypeInfo::name(&<#raw_id as ::sqlx::Type<#backing_db>>::type_info()).to_string(),
                column_type: ::storage_noodle_sql::schema::ColumnType::PrimaryKey,
            }
        }
    };

    // The table name.
    let name = syn::LitStr::new(&ident.to_string(), proc_macro2::Span::call_site());

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::schema::MakeSqlTable<#backing_db> for #ident #type_generics #where_clause {
            fn table() -> ::storage_noodle_sql::schema::SqlTable {
                let columns = ::std::vec![
                    #sql_rows_punctuated,
                    #id_column
                ];

                ::storage_noodle_sql::schema::SqlTable {
                    name: #name.to_string(),
                    columns,
                }
            }
        }
    }
}
