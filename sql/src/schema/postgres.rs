/// Generate a CREATE TABLE schema query for postgres.
pub fn generate_schema(table: &super::SqlTable) -> String {
    let columns = table.columns.iter().map(generate_row).collect::<Vec<_>>();

    format!("CREATE TABLE {} ({});", table.name, columns.join(", "))
}

/// Generate the table rows, types, and constraints for postgres.
fn generate_row(sql_column: &super::SqlColumn) -> String {
    let super::SqlColumn {
        name,
        ty,
        column_type,
    } = sql_column;

    let ty = process_type(ty.clone());

    match column_type {
        super::ColumnType::Data => format!("{name} {ty}"),
        super::ColumnType::PrimaryKey => format!("{name} {ty} PRIMARY KEY"),
    }
}

/// Process types.
fn process_type(mut ty: String) -> String {
    // HACK: For some reason, some types have '"' surrounding them, which doesn't actually work in
    // queries...
    ty.retain(|c| c != '"');
    ty
}
