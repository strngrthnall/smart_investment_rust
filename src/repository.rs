use crate::{
    app::AppState,
    models::{Asset, UserRecord},
};
use axum::{extract::FromRequestParts, http::request::Parts};
use sqlx::PgPool;
use std::convert::Infallible;
use crate::models::OwnedAsset;

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value, updated_at 
                FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
                VALUES ($1, $2)
                RETURNING id, name, unit_value, updated_at;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
                SET name=COALESCE($2, name),
                unit_value=COALESCE($3, unit_value)
                WHERE id=$1
                RETURNING id, name, unit_value, updated_at;",
            asset_id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_asset_names(&self) -> sqlx::Result<Vec<String>> {
        sqlx::query_scalar!("SELECT name FROM assets;")
            .fetch_all(&self.db)
            .await
    }

    pub async fn update_asset_value_by_name(&self, name: &str, unit_value: f64) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
                SET unit_value = $2,
                    updated_at = NOW()
                WHERE name = $1
                RETURNING id, name, unit_value, updated_at;",
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
                VALUES ($1, $2)
                RETURNING id, username, password_hash;",
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash FROM users WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn get_user_by_id(&self, id: i64) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash FROM users WHERE id = $1;",
            id
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        sqlx::query_as!(
            OwnedAsset,
            r#"
            SELECT
                asset.id,
                asset.name,
                asset.unit_value,
                SUM((asset.unit_value - owned_asset.bought_for) * owned_asset.quantity_owned) AS "value_delta!",
                SUM(owned_asset.quantity_owned) AS "quantity_owned!",
                JSON_AGG(
                    JSON_BUILD_OBJECT(
                        'bought_at', owned_asset.timestamp,
                        'bought_for', owned_asset.bought_for,
                        'quantity_bought', owned_asset.quantity_owned,
                        'value_delta', (asset.unit_value - owned_asset.bought_for) * owned_asset.quantity_owned
                    )
                ) AS "purchase_history!: _"
            FROM assets AS asset
            JOIN owned_assets AS owned_asset ON owned_asset.asset_id = asset.id
            WHERE owned_asset.user_id = $1
            GROUP BY asset.id;
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn insert_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
        unit_value: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO owned_assets
                (user_id, asset_id, quantity_owned, bought_for)
                VALUES ($1, $2, $3, $4);",
            user_id, asset_id, quantity, unit_value
        )
            .execute(&self.db)
            .await?;

        Ok(())
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
