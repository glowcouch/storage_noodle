use storage_noodle_traits::Create;

#[tokio::test]
async fn main() {
    // Set up the backing storage.
    let db_pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap();

    // Schema generation is not implemented yet
    sqlx::query(
        "
            CREATE TABLE Cookie (
            flavour TEXT,
            Id INTEGER PRIMARY Key
            );
        ",
    )
    .execute(&db_pool)
    .await
    .unwrap();

    let backing: storage_noodle_sql::SqlBacking<_, u64> =
        storage_noodle_sql::SqlBacking::new(db_pool);

    // Create a new record.
    Cookie {
        flavour: "chocolate".to_string(),
    }
    .create(&backing)
    .await
    .unwrap();
}

#[derive(storage_noodle_sql::Create)]
#[config_noodle_sql(sqlx::sqlite::Sqlite, u64)]
struct Cookie {
    flavour: String,
}
