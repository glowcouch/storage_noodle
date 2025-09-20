use proc_macro2::TokenStream;

/// Holds the arguments of a `config_noodle_sql` attribute.
pub struct SqlAttr {
    pub backing_db: syn::Type,
    pub raw_id: syn::Type,
}

impl SqlAttr {
    /// Try to convert from an attribute to a [`SqlAttr`].
    ///
    /// Returns [`None`] if the error is not a `config_noodle_sql` attribute.
    pub fn from_attribute(attr: syn::Attribute) -> syn::Result<Option<SqlAttr>> {
        // Check that the ident is correct.
        if attr.path().is_ident("config_noodle_sql") {
            // Parse the inner punctuated arguments.
            let input: syn::punctuated::Punctuated<syn::Type, syn::Token![,]> = attr
                .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                .map_err(|e| syn::Error::new_spanned(&attr, e.to_string()))?;

            // Check that there are exactly two arguments.
            if input.len() != 2 {
                let error = syn::Error::new_spanned(
                    &attr,
                    "must be in the format `config_noodle_sql(backing_db, raw_id)`",
                );

                Err(error)
            } else {
                // Return the arguments.
                Ok(Some(Self {
                    backing_db: input[0].clone(),
                    raw_id: input[1].clone(),
                }))
            }
        } else {
            Ok(None)
        }
    }

    /// Extract [`SqlAttr`]s from an item.
    ///
    /// Returns a compile error if there are none. Returns
    /// multiple errors if there were multiple errors during parsing.
    pub fn from_item(item: syn::ItemStruct) -> Result<Vec<crate::attr::SqlAttr>, TokenStream> {
        // Parse the attributes.
        let parsed_attrs = item
            .attrs
            .clone()
            .into_iter()
            .map(crate::attr::SqlAttr::from_attribute);

        // A list of errors, if any.
        let errors: Vec<_> = parsed_attrs
            .clone()
            .filter_map(|result| result.err())
            .collect();

        // Return errors, if any.
        if !errors.is_empty() {
            return Err(errors.iter().map(|e| e.to_compile_error()).collect());
        }

        // A list of the attributes we care about.
        let sql_attrs: Vec<_> = parsed_attrs
            .filter_map(|result| result.ok().flatten())
            .collect();

        // Return error if there are no attributes.
        if sql_attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                item,
                "there is no `config_noodle_sql` attribute on this struct",
            )
            .to_compile_error());
        }

        // Otherwise, return the sucessfull attributes.
        Ok(sql_attrs)
    }
}
