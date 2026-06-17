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
}

#[sqlx::test]
async fn test_get_user_by_id_not_found(db: PgPool) {
    let repository = Repository::from(db);
    let found = repository
        .get_user_by_id(999_999)
        .await
        .expect("query should succeed");
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_get_user_by_name_not_found(db: PgPool) {
    let repository = Repository::from(db);
    let found = repository
        .get_user_by_name("ghost_user")
        .await
        .expect("query should succeed");
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_update_asset_returns_none_for_missing_id(db: PgPool) {
    let repository = Repository::from(db);
    let result = repository
        .update_asset(999_999, Some("Nope".to_string()), Some(1.0))
        .await
        .expect("query should succeed");
    assert!(result.is_none());
}
