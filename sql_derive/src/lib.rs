mod sql_backing;

/// Derives `Create` `Read` `Update` `Delete<SqlBacking>`
#[proc_macro_derive(SqlBacking)]
pub fn sql_backing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    sql_backing::sql_backing(input.into()).into()
}
