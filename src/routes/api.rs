use axum::{Json, Router, routing::get};
use serde::Deserialize;

use crate::{
    app::AppState, auth::admin::Admin, error::AppError, models::Asset, repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/assets",
            get(list_assets).post(create_asset).patch(update_asset),
        )
        .route("/assets/sync", axum::routing::post(sync_assets))
}

use std::collections::HashMap;

#[derive(Deserialize)]
struct AwesomeApiResponseItem {
    bid: String,
}

pub async fn sync_assets_with_api(repository: &Repository) -> Result<(), AppError> {
    let assets = repository.list_assets().await?;

    let now = time::OffsetDateTime::now_utc();
    let limit = time::Duration::hours(24);

    // Check if at least one asset needs an update
    let needs_update = assets.iter().any(|asset| {
        let diff = now - asset.updated_at;
        diff > limit
    });

    if !needs_update {
        return Ok(());
    }

    let data: HashMap<String, AwesomeApiResponseItem> = if std::env::var("INTEGRATION_TEST").is_ok() {
        let mock_json = r#"{
            "USDBRL": { "bid": "5.60" },
            "BTCBRL": { "bid": "330000.0" },
            "ETHBRL": { "bid": "15000.0" }
        }"#;
        serde_json::from_str(mock_json).unwrap()
    } else {
        let url = "https://economia.awesomeapi.com.br/json/last/USD-BRL,BTC-BRL,ETH-BRL";
        let response = reqwest::get(url).await?;
        response.json().await?
    };

    for asset in assets {
        let diff = now - asset.updated_at;
        if diff <= limit {
            // Only update if difference between stored date and current date is greater than 24 hours
            continue;
        }

        let key = match asset.name.as_str() {
            "Dólar" => Some("USDBRL"),
            "Bitcoin" => Some("BTCBRL"),
            "Ethereum" => Some("ETHBRL"),
            _ => None,
        };

        if let Some(api_key) = key {
            if let Some(item) = data.get(api_key) {
                if let Ok(new_price) = item.bid.parse::<f64>() {
                    repository.update_asset_value_by_name(&asset.name, new_price).await?;
                }
            }
        } else if asset.name == "Real" {
            repository.update_asset_value_by_name(&asset.name, 1.0).await?;
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn sync_assets(repository: Repository) -> Result<Json<String>, AppError> {
    sync_assets_with_api(&repository).await?;
    Ok(Json("Assets synced successfully".to_string()))
}

#[tracing::instrument(skip_all)]
pub async fn list_assets(repository: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repository.list_assets().await?;
    Ok(Json(assets))
}

#[derive(Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub unit_value: f64,
}

#[tracing::instrument(skip_all)]
pub async fn create_asset(
    _: Admin,
    repository: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let new_asset = repository
        .create_asset(request.name, request.unit_value)
        .await?;
    Ok(Json(new_asset))
}

#[derive(Deserialize)]
pub struct UpdateAssetRequest {
    pub id: i64,
    pub name: Option<String>,
    pub unit_value: Option<f64>,
}

#[tracing::instrument(skip_all)]
pub async fn update_asset(
    _: Admin,
    repository: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    match repository
        .update_asset(request.id, request.name, request.unit_value)
        .await?
    {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExists),
    }
}
