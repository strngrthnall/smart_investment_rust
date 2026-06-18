use smart_investment_rust::repository::Repository;
use sqlx::PgPool;

#[sqlx::test]
async fn test_get_user_by_id_found(db: PgPool) {
    let repository = Repository::from(db);
    let created = repository
        .add_user("lookup_user", "hash")
        .await
        .expect("should insert user");

    let found = repository
        .get_user_by_id(created.id)
        .await
        .expect("query should succeed")
        .expect("user should exist");

    assert_eq!(found.id, created.id);
    assert_eq!(found.username, "lookup_user");
    insta::assert_debug_snapshot!(found);
}

#[sqlx::test]
async fn test_get_user_by_id_not_found(db: PgPool) {
    let repository = Repository::from(db);
    let found = repository
        .get_user_by_id(999_999)
        .await
        .expect("query should succeed");
    assert!(found.is_none());
    insta::assert_debug_snapshot!(found);
}

#[sqlx::test]
async fn test_get_user_by_name_not_found(db: PgPool) {
    let repository = Repository::from(db);
    let found = repository
        .get_user_by_name("ghost_user")
        .await
        .expect("query should succeed");
    assert!(found.is_none());
    insta::assert_debug_snapshot!(found);
}

#[sqlx::test]
async fn test_update_asset_returns_none_for_missing_id(db: PgPool) {
    let repository = Repository::from(db);
    let result = repository
        .update_asset(999_999, Some("Nope".to_string()), Some(1.0))
        .await
        .expect("query should succeed");
    assert!(result.is_none());
    insta::assert_json_snapshot!(result);
}

#[sqlx::test]
async fn test_list_asset_names(db: PgPool) {
    let repository = Repository::from(db);
    repository
        .create_asset("Bitcoin".to_string(), 10.0)
        .await
        .expect("should insert asset");
    repository
        .create_asset("Ethereum".to_string(), 20.0)
        .await
        .expect("should insert asset");

    let mut names = repository
        .list_asset_names()
        .await
        .expect("query should succeed");
    // The query has no ORDER BY, so sort for a deterministic assertion.
    names.sort();
    assert_eq!(names, ["Bitcoin".to_string(), "Ethereum".to_string()]);
    insta::assert_json_snapshot!(names);
}

#[sqlx::test]
async fn test_update_asset_value_by_name_updates_price(db: PgPool) {
    let repository = Repository::from(db);
    let original = repository
        .create_asset("Bitcoin".to_string(), 10.0)
        .await
        .expect("should insert asset");

    let updated = repository
        .update_asset_value_by_name("Bitcoin", 350_000.0)
        .await
        .expect("query should succeed")
        .expect("asset should exist");

    assert_eq!(updated.id, original.id);
    assert_eq!(updated.name, "Bitcoin");
    assert_eq!(updated.unit_value, 350_000.0);
    // The update bumps `updated_at` via NOW().
    assert!(updated.updated_at >= original.updated_at);
    insta::assert_json_snapshot!(updated, {
        ".updated_at" => "[datetime]"
    });
}

#[sqlx::test]
async fn test_update_asset_value_by_name_missing(db: PgPool) {
    let repository = Repository::from(db);
    let result = repository
        .update_asset_value_by_name("Nonexistent", 1.0)
        .await
        .expect("query should succeed");
    assert!(result.is_none());
    insta::assert_json_snapshot!(result);
}

#[sqlx::test]
async fn test_insert_and_list_owned_asset(db: PgPool) {
    let repository = Repository::from(db);
    let user = repository
        .add_user("owner", "hash")
        .await
        .expect("should insert user");
    let asset = repository
        .create_asset("Bitcoin".to_string(), 100.0)
        .await
        .expect("should insert asset");

    // Bought 2 units at 80 each; the asset is now worth 100.
    repository
        .insert_owned_asset(user.id, asset.id, 2.0, 80.0)
        .await
        .expect("should insert owned asset");

    let owned = repository
        .list_owned_assets(user.id)
        .await
        .expect("query should succeed");

    assert_eq!(owned.len(), 1);
    let holding = &owned[0];
    assert_eq!(holding.id, asset.id);
    assert_eq!(holding.name, "Bitcoin");
    assert_eq!(holding.quantity_owned, 2.0);
    // (current 100 - bought 80) * 2 units = 40
    assert_eq!(holding.value_delta, 40.0);

    assert_eq!(holding.purchase_history.0.len(), 1);
    let purchase = &holding.purchase_history.0[0];
    assert_eq!(purchase.bought_for, 80.0);
    assert_eq!(purchase.quantity_bought, 2.0);
    assert_eq!(purchase.value_delta, 40.0);

    // `bought_at` is a server-generated timestamp, so redact it to keep the
    // snapshot deterministic.
    insta::assert_json_snapshot!(owned, {
        "[].purchase_history[].bought_at" => "[bought_at]"
    });
}

#[sqlx::test]
async fn test_list_owned_assets_aggregates_multiple_purchases(db: PgPool) {
    let repository = Repository::from(db);
    let user = repository
        .add_user("multi_owner", "hash")
        .await
        .expect("should insert user");
    let asset = repository
        .create_asset("Ethereum".to_string(), 100.0)
        .await
        .expect("should insert asset");

    repository
        .insert_owned_asset(user.id, asset.id, 2.0, 80.0)
        .await
        .expect("first purchase should insert");
    repository
        .insert_owned_asset(user.id, asset.id, 1.0, 120.0)
        .await
        .expect("second purchase should insert");

    let owned = repository
        .list_owned_assets(user.id)
        .await
        .expect("query should succeed");

    // Both purchases are grouped under the same asset.
    assert_eq!(owned.len(), 1);
    let holding = &owned[0];
    assert_eq!(holding.quantity_owned, 3.0);
    // (100 - 80) * 2 + (100 - 120) * 1 = 40 - 20 = 20
    assert_eq!(holding.value_delta, 20.0);
    assert_eq!(holding.purchase_history.0.len(), 2);

    // The purchase_history array order from JSON_AGG is not guaranteed, so
    // snapshot the stable aggregates instead of the full object.
    insta::assert_debug_snapshot!((
        holding.quantity_owned,
        holding.value_delta,
        holding.purchase_history.0.len(),
    ));
}

#[sqlx::test]
async fn test_list_owned_assets_empty(db: PgPool) {
    let repository = Repository::from(db);
    let user = repository
        .add_user("no_assets", "hash")
        .await
        .expect("should insert user");

    let owned = repository
        .list_owned_assets(user.id)
        .await
        .expect("query should succeed");
    assert!(owned.is_empty());
    insta::assert_json_snapshot!(owned);
}

#[sqlx::test]
async fn test_insert_owned_asset_invalid_foreign_key(db: PgPool) {
    let repository = Repository::from(db);
    // No user/asset with these ids exist, so the FK constraint must reject it.
    let err = repository
        .insert_owned_asset(999_999, 999_999, 1.0, 1.0)
        .await
        .unwrap_err();
    // Snapshot the violated constraint name rather than the raw message, which
    // embeds a Postgres-build-specific source line number.
    let constraint = err
        .as_database_error()
        .and_then(|db_err| db_err.constraint())
        .map(str::to_string);
    insta::assert_debug_snapshot!(constraint);
}
