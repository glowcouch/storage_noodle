use storage_noodle_traits::{Create, Read, Update};

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

    let backing: storage_noodle_sql::SqlBacking<_, u32> =
        storage_noodle_sql::SqlBacking::new(db_pool);

    // Create a cookie
    let cookie = Cookie {
        flavour: "chocolate".to_string(),
    };

    // Create a new record.
    let cookie_id = cookie.create(&backing).await.unwrap();

    // Read the record back from the db.
    let returned_cookie = Cookie::read(&backing, &cookie_id).await.unwrap();

    // Check that the cookie did not get altered.
    assert_eq!(returned_cookie, Some(cookie));

    let new_cookie = Cookie {
        flavour: "strawberry".to_string(),
    };

    // Update the record.
    new_cookie.update(&backing, &cookie_id).await.unwrap();

    // Read the record back from the db.
    let returned_cookie = Cookie::read(&backing, &cookie_id).await.unwrap();

    // Check that the cookie got updated correctly.
    assert_eq!(returned_cookie, Some(new_cookie));
}

#[derive(
    Debug,
    PartialEq,
    storage_noodle_sql::Create,
    storage_noodle_sql::Read,
    storage_noodle_sql::Update,
    sqlx::FromRow,
)]
#[config_noodle_sql(sqlx::sqlite::Sqlite, u32)]
struct Cookie {
    flavour: String,
}
