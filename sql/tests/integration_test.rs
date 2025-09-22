use storage_noodle_traits::{Create, Delete, Read, Update};

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
            recipe INTEGER,
            Id INTEGER PRIMARY Key
            );

            CREATE TABLE Recipe (
            ingredients TEXT,
            Id INTEGER PRIMARY Key
            );
        ",
    )
    .execute(&db_pool)
    .await
    .unwrap();

    let backing: storage_noodle_sql::SqlBacking<_, u32> =
        storage_noodle_sql::SqlBacking::new(db_pool);

    // Create a recipe
    let choco_recipe_id = Recipe {
        ingredients: "eggs, flour, sugar, butter, chocolate chips".to_string(),
    }
    .create(&backing)
    .await
    .unwrap();

    // Create a cookie
    let cookie = Cookie {
        flavour: "chocolate".to_string(),
        recipe: choco_recipe_id,
    };

    // Create a new record.
    let cookie_id = cookie.create(&backing).await.unwrap();

    // Read the record back from the db.
    let returned_cookie = Cookie::read(&backing, &cookie_id).await.unwrap();

    // Check that the cookie did not get altered.
    assert_eq!(returned_cookie, Some(cookie));

    // Create a recipe
    let strawb_recipe_id = Recipe {
        ingredients: "eggs, flour, sugar, butter, strawberries".to_string(),
    }
    .create(&backing)
    .await
    .unwrap();

    let new_cookie = Cookie {
        flavour: "strawberry".to_string(),
        recipe: strawb_recipe_id,
    };

    // Update the record.
    new_cookie.update(&backing, &cookie_id).await.unwrap();

    // Read the record back from the db.
    let returned_cookie = Cookie::read(&backing, &cookie_id).await.unwrap().unwrap();

    // Check that the cookie got updated correctly.
    assert_eq!(returned_cookie, new_cookie);

    // Delete the record.
    let deleted_cookie = Cookie::delete(&backing, &cookie_id).await.unwrap().unwrap();

    // Check that the cookie we deleted has not changed.
    assert_eq!(returned_cookie, deleted_cookie);

    // Try to read the record again.
    let should_be_none = Cookie::read(&backing, &cookie_id).await.unwrap();

    // Check that the record didn't exist.
    assert!(should_be_none.is_none());
}

#[derive(
    Debug,
    PartialEq,
    storage_noodle_sql::Create,
    storage_noodle_sql::Read,
    storage_noodle_sql::Update,
    storage_noodle_sql::Delete,
    sqlx::FromRow,
)]
#[config_noodle_sql(sqlx::sqlite::Sqlite, u32)]
#[config_noodle_raw_id(RawId)]
struct Cookie<RawId> {
    flavour: String,
    recipe: storage_noodle_traits::AssocId<Recipe, RawId>,
}

#[derive(
    Debug,
    PartialEq,
    storage_noodle_sql::Create,
    storage_noodle_sql::Read,
    storage_noodle_sql::Update,
    storage_noodle_sql::Delete,
    sqlx::FromRow,
)]
#[config_noodle_sql(sqlx::sqlite::Sqlite, u32)]
struct Recipe {
    ingredients: String,
}
