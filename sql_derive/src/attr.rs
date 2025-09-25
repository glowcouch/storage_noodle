use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ItemStruct;

/// Runs `func` on each attribute and returns a list of impl blocks.
pub fn for_each_attr(
    item: &syn::ItemStruct,
    func: impl Fn(&syn::ItemStruct, &syn::Type, &syn::Type) -> TokenStream,
) -> TokenStream {
    // Get the sql attribute(s). Error if there are none.
    let sql_attrs = match crate::attr::SqlAttr::from_item(item.clone()) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // Run `create_impl` on the attributes and return the resulting impl block(s).
    sql_attrs
        .iter()
        .map(|crate::attr::SqlAttr { backing_db, raw_id }| func(item, backing_db, raw_id))
        .collect()
}

/// Holds the arguments of a `storage_noodle_sql` attribute.
pub struct SqlAttr {
    /// The backing database type (typically a type implementing `sqlx::Database`).
    pub backing_db: syn::Type,

    /// The raw id type.
    pub raw_id: syn::Type,
}

impl SqlAttr {
    /// Try to convert from an attribute to a [`SqlAttr`].
    ///
    /// Returns [`None`] if the error is not a `storage_noodle_sql` attribute.
    pub fn from_attribute(attr: &syn::Attribute) -> syn::Result<Option<Self>> {
        // Check that the ident is correct.
        if attr.path().is_ident("storage_noodle_sql") {
            // Parse the inner punctuated arguments.
            let input: syn::punctuated::Punctuated<syn::Type, syn::Token![,]> = attr
                .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                .map_err(|e| syn::Error::new_spanned(attr, e.to_string()))?;

            // Check that there are exactly two arguments.
            if input.len() == 2 {
                // Return the arguments.
                Ok(Some(Self {
                    backing_db: input[0].clone(),
                    raw_id: input[1].clone(),
                }))
            } else {
                let error = syn::Error::new_spanned(
                    attr,
                    "must be in the format `storage_noodle_sql(backing_db, raw_id)`",
                );

                Err(error)
            }
        } else {
            Ok(None)
        }
    }

    /// Extract [`SqlAttr`]s from an item.
    ///
    /// Returns a compile error if there are none. Returns
    /// multiple errors if there were multiple errors during parsing.
    pub fn from_item(item: syn::ItemStruct) -> Result<Vec<Self>, TokenStream> {
        // Parse the attributes.
        let parsed_attrs = item
            .attrs
            .clone()
            .into_iter()
            .map(|a| Self::from_attribute(&a));

        // A list of errors, if any.
        let errors: Vec<_> = parsed_attrs
            .clone()
            .filter_map(core::result::Result::err)
            .collect();

        // Return errors, if any.
        if !errors.is_empty() {
            return Err(errors.iter().map(syn::Error::to_compile_error).collect());
        }

        // A list of the attributes we care about.
        let sql_attrs: Vec<_> = parsed_attrs
            .filter_map(|result| result.ok().flatten())
            .collect();

        // Return error if there are no attributes.
        if sql_attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                item,
                "there is no `storage_noodle_sql` attribute on this struct",
            )
            .to_compile_error());
        }

        // Otherwise, return the sucessfull attributes.
        Ok(sql_attrs)
    }
}

/// Extracts a type generic from the `storage_noodle_raw_id` attribute.
pub fn raw_id_attr(item: &ItemStruct) -> Option<Result<syn::Ident, syn::Error>> {
    item.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("storage_noodle_raw_id") {
            Some(attr.parse_args())
        } else {
            None
        }
    })
}

/// Similar to [`syn::Generics::split_for_impl`]. Returns (impl generics, type generics, where clause). Aditionally, it turns the
/// `to_replace` generic into the concrete type `concrete` - removing it from the impl generics and
/// where clause.
pub fn split_for_impl_make_concrete(
    generics: &syn::Generics,
    to_replace: &[syn::Ident],
    concrete: &syn::Type,
) -> (TokenStream, TokenStream, Option<TokenStream>) {
    // The generics we want to replace.
    let matched_generics = generics
        .params
        .iter()
        .filter(|param| {
            if let syn::GenericParam::Type(ty) = param {
                // Does the generic match?
                to_replace.contains(&ty.ident)
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    // Impl & type generics without the annotated generics.
    let filtered_generics = syn::Generics {
        params: generics
            .params
            .clone()
            .into_iter()
            .filter(|param| !matched_generics.contains(&param))
            .collect(),
        ..generics.clone()
    };

    // Filtered generics split for impl.
    let filtered_impl_generics = filtered_generics.split_for_impl().0;

    // Filtered generics split for where clause.
    let filtered_where_clause = filtered_generics.split_for_impl().2;

    // Substitute the type generics.
    let substituted_type_generics = generics.params.iter().map(|param|
        // "Render" the generics to put in the type. Like this: `SomeType<A, B, C>`.
        match param {
        syn::GenericParam::Lifetime(lifetime_param) => lifetime_param.lifetime.to_token_stream(),
        syn::GenericParam::Type(type_param) => {
            // If the generic parameter is annotated.
            if matched_generics.contains(&param) {
                // Replace it with the concrete type.
                concrete.to_token_stream()
            } else {
                type_param.ident.to_token_stream()
            }
        }
        syn::GenericParam::Const(const_param) => const_param.ident.to_token_stream(),
    });

    // Comma punctuated list of type generics.
    let type_generics: TokenStream =
        itertools::Itertools::intersperse(substituted_type_generics, quote::quote! {,}).collect();

    (
        filtered_impl_generics.to_token_stream(),
        quote::quote! {<#type_generics>},
        filtered_where_clause.map(quote::ToTokens::to_token_stream),
    )
}

/// Produces split generics from a struct using [`split_for_impl_make_concrete`] and [`raw_id_attr`]. Returns (impl generics, type generics, where clause).
pub fn split_generics_with_raw_id_attr(
    item: &ItemStruct,
    raw_id: &syn::Type,
) -> Result<(TokenStream, TokenStream, Option<TokenStream>), syn::Error> {
    if let Some(to_replace) = raw_id_attr(item) {
        // Use [`split_for_impl_make_concrete`].
        Ok(split_for_impl_make_concrete(
            &item.generics,
            &[to_replace?],
            raw_id,
        ))
    } else {
        // Use syn's split_for_impl.
        let (r#impl, r#type, r#where) = item.generics.split_for_impl();
        Ok((
            r#impl.to_token_stream(),
            r#type.to_token_stream(),
            r#where.map(quote::ToTokens::to_token_stream),
        ))
    }
}

/// Replaces type generics in a type with a concrete type.
pub fn make_concrete(ty: &syn::Type, replace: &syn::Ident, with: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            if type_path.path.is_ident(replace) {
                with.clone()
            } else {
                // Check generic params.
                let mut new = type_path.clone();

                // For each angle bracketed generic parameter.
                new.path.segments.iter_mut().for_each(|segment| {
                    if let syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) =
                        &mut segment.arguments
                    {
                        angle_bracketed_generic_arguments
                            .args
                            .iter_mut()
                            .for_each(|arg| {
                                if let syn::GenericArgument::Type(generic_type) = arg {
                                    // Run `make_concrete` on the generic parameter.
                                    *generic_type = make_concrete(generic_type, replace, with);
                                }
                            });
                    }
                });

                syn::Type::Path(new)
            }
        }
        _ => ty.clone(),
    }
}
