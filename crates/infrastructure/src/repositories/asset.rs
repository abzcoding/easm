use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{models::Asset, traits::AssetRepository, Result};
use shared::types::{AssetStatus, AssetType, ID};
use sqlx::PgPool;

/// PostgreSQL implementation of the Asset Repository
pub struct PgAssetRepository {
    pool: PgPool,
}

impl PgAssetRepository {
    /// Create a new PgAssetRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
        // Convert DateTime types for database operation
        let first_seen = to_offset_datetime(asset.first_seen);
        let last_seen = to_offset_datetime(asset.last_seen);
        let created_at = to_offset_datetime(asset.created_at);
        let updated_at = to_offset_datetime(asset.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO assets (id, organization_id, asset_type, value, status, first_seen, last_seen, created_at, updated_at, attributes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
            "#,
            asset.id,
            asset.organization_id,
            asset.asset_type as AssetType,
            asset.value,
            asset.status as AssetStatus,
            first_seen,
            last_seen,
            created_at,
            updated_at,
            asset.attributes
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Asset {
            id: record.id,
            organization_id: record.organization_id,
            asset_type: record.asset_type,
            value: record.value,
            status: record.status.expect("Asset status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            attributes: record
                .attributes
                .expect("Asset attributes should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn get_asset(&self, id: ID) -> Result<Asset> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
            FROM assets
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Asset {
            id: record.id,
            organization_id: record.organization_id,
            asset_type: record.asset_type,
            value: record.value,
            status: record.status.expect("Asset status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            attributes: record
                .attributes
                .expect("Asset attributes should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn update_asset(&self, asset: &Asset) -> Result<Asset> {
        // Convert DateTime types for database operation
        let first_seen = to_offset_datetime(asset.first_seen);
        let last_seen = to_offset_datetime(asset.last_seen);
        let updated_at = to_offset_datetime(asset.updated_at);

        let record = sqlx::query!(
            r#"
            UPDATE assets
            SET organization_id = $2, asset_type = $3, value = $4, status = $5, first_seen = $6, last_seen = $7, updated_at = $8, attributes = $9
            WHERE id = $1
            RETURNING id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
            "#,
            asset.id,
            asset.organization_id,
            asset.asset_type as AssetType,
            asset.value,
            asset.status as AssetStatus,
            first_seen,
            last_seen,
            updated_at,
            asset.attributes
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(Asset {
            id: record.id,
            organization_id: record.organization_id,
            asset_type: record.asset_type,
            value: record.value,
            status: record.status.expect("Asset status should not be null"),
            first_seen: from_offset_datetime(Some(record.first_seen)),
            last_seen: from_offset_datetime(Some(record.last_seen)),
            attributes: record
                .attributes
                .expect("Asset attributes should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn delete_asset(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM assets
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Asset>> {
        let assets = match (organization_id, asset_type, status) {
            (Some(org_id), Some(a_type), Some(s)) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE organization_id = $1 AND asset_type = $2 AND status = $3
                    ORDER BY value
                    LIMIT $4 OFFSET $5
                    "#,
                    org_id,
                    a_type as AssetType,
                    s as AssetStatus,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (Some(org_id), Some(a_type), None) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE organization_id = $1 AND asset_type = $2
                    ORDER BY value
                    LIMIT $3 OFFSET $4
                    "#,
                    org_id,
                    a_type as AssetType,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (Some(org_id), None, Some(s)) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE organization_id = $1 AND status = $2
                    ORDER BY value
                    LIMIT $3 OFFSET $4
                    "#,
                    org_id,
                    s as AssetStatus,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (None, Some(a_type), Some(s)) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE asset_type = $1 AND status = $2
                    ORDER BY value
                    LIMIT $3 OFFSET $4
                    "#,
                    a_type as AssetType,
                    s as AssetStatus,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (Some(org_id), None, None) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE organization_id = $1
                    ORDER BY value
                    LIMIT $2 OFFSET $3
                    "#,
                    org_id,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (None, Some(a_type), None) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE asset_type = $1
                    ORDER BY value
                    LIMIT $2 OFFSET $3
                    "#,
                    a_type as AssetType,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (None, None, Some(s)) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    WHERE status = $1
                    ORDER BY value
                    LIMIT $2 OFFSET $3
                    "#,
                    s as AssetStatus,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            (None, None, None) => {
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, asset_type as "asset_type: AssetType", value, status as "status: AssetStatus", first_seen, last_seen, created_at, updated_at, attributes
                    FROM assets
                    ORDER BY value
                    LIMIT $1 OFFSET $2
                    "#,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Asset {
                        id: record.id,
                        organization_id: record.organization_id,
                        asset_type: record.asset_type,
                        value: record.value,
                        status: record.status.expect("Asset status should not be null"),
                        first_seen: from_offset_datetime(Some(record.first_seen)),
                        last_seen: from_offset_datetime(Some(record.last_seen)),
                        attributes: record
                            .attributes
                            .expect("Asset attributes should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
        };

        Ok(assets)
    }

    async fn count_assets(
        &self,
        organization_id: Option<ID>,
        asset_type: Option<AssetType>,
        status: Option<AssetStatus>,
    ) -> Result<usize> {
        let count: Option<i64> = match (organization_id, asset_type, status) {
            (Some(org_id), Some(a_type), Some(s)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE organization_id = $1 AND asset_type = $2 AND status = $3
                    "#,
                    org_id,
                    a_type as AssetType,
                    s as AssetStatus
                )
                .fetch_one(&self.pool)
                .await?
            }
            (Some(org_id), Some(a_type), None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE organization_id = $1 AND asset_type = $2
                    "#,
                    org_id,
                    a_type as AssetType,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (Some(org_id), None, Some(s)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE organization_id = $1 AND status = $2
                    "#,
                    org_id,
                    s as AssetStatus,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(a_type), Some(s)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE asset_type = $1 AND status = $2
                    "#,
                    a_type as AssetType,
                    s as AssetStatus,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (Some(org_id), None, None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE organization_id = $1
                    "#,
                    org_id,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (None, Some(a_type), None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE asset_type = $1
                    "#,
                    a_type as AssetType,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (None, None, Some(s)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    WHERE status = $1
                    "#,
                    s as AssetStatus,
                )
                .fetch_one(&self.pool)
                .await?
            }
            (None, None, None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM assets
                    "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(count.unwrap_or(0) as usize)
    }
}
