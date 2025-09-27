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
    match column_type {
        super::ColumnType::Data => format!("{name} {ty}"),
        super::ColumnType::PrimaryKey => format!("{name} {ty} PRIMARY KEY"),
    }
}
