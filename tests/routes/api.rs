use axum::Json;
use smart_investment_rust::{
    auth::admin::Admin,
    error::AppError,
    repository::Repository,
    routes::api::{
        create_asset, list_assets, sync_assets, sync_assets_with_api, update_asset,
        CreateAssetRequest, UpdateAssetRequest,
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

    insta::assert_json_snapshot!(new_asset, {
        ".updated_at" => "[datetime]"
    });
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_list_assets(db: PgPool) {
    let Json(assets) = list_assets(db.into()).await.expect("success");

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "Bitcoin");

    insta::assert_json_snapshot!(assets, {
        "[].updated_at" => "[datetime]"
    });
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

    insta::assert_json_snapshot!(update_asset, {
        ".updated_at" => "[datetime]"
    });
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
    insta::assert_json_snapshot!(updated, {
        ".updated_at" => "[datetime]"
    });
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
    insta::assert_json_snapshot!(updated, {
        ".updated_at" => "[datetime]"
    });
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
    insta::assert_json_snapshot!(updated, {
        ".updated_at" => "[datetime]"
    });
}

#[sqlx::test(fixtures("bitcoin_asset"))]
async fn test_sync_assets(db: PgPool) {
    let repository = db.into();
    
    unsafe {
        std::env::set_var("INTEGRATION_TEST", "1");
    }
    // Call the sync function
    let res = smart_investment_rust::routes::api::sync_assets_with_api(&repository).await;
    unsafe {
        std::env::remove_var("INTEGRATION_TEST");
    }

    res.expect("sync should succeed");

    // Fetch the updated assets list
    let Json(assets) = list_assets(repository)
        .await
        .expect("list should succeed");

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].name, "Bitcoin");
    // The price should be updated from the default 10.0 to the live API cotação (e.g. > 300,000.0 BRL)
    assert!(assets[0].unit_value > 10.0);
}

#[sqlx::test]
async fn test_sync_assets_with_api_skips_fresh_assets(db: PgPool) {
    let repository: Repository = db.into();
    // A freshly created asset has updated_at = NOW(), inside the 24h window, so
    // sync is a no-op: no network call and the price stays unchanged.
    repository
        .create_asset("Bitcoin".to_string(), 10.0)
        .await
        .expect("should insert asset");

    sync_assets_with_api(&repository)
        .await
        .expect("sync should be a no-op for fresh assets");

    let Json(assets) = list_assets(repository).await.expect("list should succeed");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].unit_value, 10.0);
}

#[sqlx::test]
async fn test_sync_assets_handler_returns_message(db: PgPool) {
    let repository: Repository = db.into();
    // Fresh asset keeps sync on the no-op path (no network), exercising the
    // handler wrapper end-to-end.
    repository
        .create_asset("Bitcoin".to_string(), 10.0)
        .await
        .expect("should insert asset");

    let Json(message) = sync_assets(repository)
        .await
        .expect("sync handler should succeed");
    assert_eq!(message, "Assets synced successfully");
    insta::assert_json_snapshot!(message);
}
