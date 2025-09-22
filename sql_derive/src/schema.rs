use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::for_each_attr;

pub fn sql_table(item: syn::ItemStruct) -> TokenStream {
    for_each_attr(item, sql_table_impl)
}

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
                ::storage_noodle_sql::schema::SqlRow {
                    name: #name.to_string(),
                    ty: ::sqlx::TypeInfo::name(&<#processed_ty as ::sqlx::Type<#backing_db>>::type_info()).to_string(),
                }
            }
        });

    let sql_rows_punctuated =
        itertools::Itertools::intersperse(sql_rows, quote! {,}).collect::<TokenStream>();

    // Implement the trait.
    quote! {
        impl #impl_generics ::storage_noodle_sql::schema::SqlTable<#backing_db> for #ident #type_generics #where_clause {
            fn rows() -> ::std::vec::Vec<::storage_noodle_sql::schema::SqlRow> {
                ::std::vec![#sql_rows_punctuated]
            }
        }
    }
}
