use axum::Json;
use smart_investment_rust::{
    auth::admin::Admin,
    error::AppError,
    routes::api::{
        create_asset, list_assets, update_asset, CreateAssetRequest, UpdateAssetRequest,
    },
};
use sqlx::PgPool;

#[sqlx::test]
async fn test_create_asset(db: PgPool) {
    let request = CreateAssetRequest {
        name: "Bitcoin".to_string(),
        unit_value: 10.0,
    };

    let Json(new_asset) = create_asset(Admin, db.into(), Json(request))
        .await
        .expect("success");

    assert_eq!(new_asset.id, 1);
    assert_eq!(new_asset.name, "Bitcoin");
    assert_eq!(new_asset.unit_value, 10.0);

    insta::assert_json_snapshot!(new_asset);
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_list_assets(db: PgPool) {
    let Json(assets) = list_assets(db.into()).await.expect("success");

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "Bitcoin");

    insta::assert_json_snapshot!(assets);
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_update_asset(db: PgPool) {
    let request = UpdateAssetRequest {
        id: 1,
        name: Some("Ethereum".to_string()),
        unit_value: Some(20.0),
    };

    let Json(update_asset) = update_asset(Admin, db.into(), Json(request))
        .await
        .expect("success");

    assert_eq!(update_asset.id, 1);
    assert_eq!(update_asset.name, "Ethereum");
    assert_eq!(update_asset.unit_value, 20.0);

    insta::assert_json_snapshot!(update_asset);
}

#[sqlx::test]
async fn test_list_assets_empty(db: PgPool) {
    let Json(assets) = list_assets(db.into()).await.expect("success");
    assert!(assets.is_empty());
    insta::assert_json_snapshot!(assets);
}

#[sqlx::test]
async fn test_update_asset_not_found(db: PgPool) {
    let request = UpdateAssetRequest {
        id: 999,
        name: Some("Ghost".to_string()),
        unit_value: Some(1.0),
    };

    let err = match update_asset(Admin, db.into(), Json(request)).await {
        Ok(_) => panic!("update of a missing asset should fail"),
        Err(err) => err,
    };
    assert!(matches!(err, AppError::AssetDoesNotExists));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_update_asset_partial_name_only(db: PgPool) {
    // Only the name changes; COALESCE must keep the original unit_value.
    let request = UpdateAssetRequest {
        id: 1,
        name: Some("Litecoin".to_string()),
        unit_value: None,
    };

    let Json(updated) = update_asset(Admin, db.into(), Json(request))
        .await
        .expect("success");
    assert_eq!(updated.name, "Litecoin");
    assert_eq!(updated.unit_value, 10.0);
    insta::assert_json_snapshot!(updated);
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_update_asset_partial_value_only(db: PgPool) {
    // Only the value changes; COALESCE must keep the original name.
    let request = UpdateAssetRequest {
        id: 1,
        name: None,
        unit_value: Some(42.0),
    };

    let Json(updated) = update_asset(Admin, db.into(), Json(request))
        .await
        .expect("success");
    assert_eq!(updated.name, "Bitcoin");
    assert_eq!(updated.unit_value, 42.0);
    insta::assert_json_snapshot!(updated);
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_update_asset_no_fields_keeps_values(db: PgPool) {
    // Both fields None; COALESCE must leave the record untouched.
    let request = UpdateAssetRequest {
        id: 1,
        name: None,
        unit_value: None,
    };

    let Json(updated) = update_asset(Admin, db.into(), Json(request))
        .await
        .expect("success");
    assert_eq!(updated.name, "Bitcoin");
    assert_eq!(updated.unit_value, 10.0);
    insta::assert_json_snapshot!(updated);
}
