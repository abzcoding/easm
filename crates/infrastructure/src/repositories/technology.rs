use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{models::Technology, traits::TechnologyRepository, Result};
use shared::types::ID;
use sqlx::PgPool;

/// PostgreSQL implementation of the Technology Repository
pub struct PgTechnologyRepository {
    pool: PgPool,
}

impl PgTechnologyRepository {
    /// Create a new PgTechnologyRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TechnologyRepository for PgTechnologyRepository {
    async fn create_technology(&self, technology: &Technology) -> Result<Technology> {
        let created_at = to_offset_datetime(technology.created_at);
        let updated_at = to_offset_datetime(technology.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO technologies (id, asset_id, name, version, category, first_seen, last_seen, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, asset_id, name, version, category, created_at, updated_at
            "#,
            technology.id,
            technology.asset_id,
            technology.name,
            technology.version,
            technology.category,
            created_at, // first_seen
            created_at, // last_seen
            created_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Technology {
            id: record.id,
            asset_id: record.asset_id,
            name: record.name,
            version: record.version,
            category: record.category,
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn get_technology(&self, id: ID) -> Result<Technology> {
        let record = sqlx::query!(
            r#"
            SELECT id, asset_id, name, version, category, created_at, updated_at
            FROM technologies
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Technology {
            id: record.id,
            asset_id: record.asset_id,
            name: record.name,
            version: record.version,
            category: record.category,
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn update_technology(&self, technology: &Technology) -> Result<Technology> {
        let updated_at = to_offset_datetime(technology.updated_at);
        let now = to_offset_datetime(chrono::Utc::now());

        let record = sqlx::query!(
            r#"
            UPDATE technologies
            SET asset_id = $2, name = $3, version = $4, category = $5, last_seen = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, asset_id, name, version, category, created_at, updated_at
            "#,
            technology.id,
            technology.asset_id,
            technology.name,
            technology.version,
            technology.category,
            now,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Technology {
            id: record.id,
            asset_id: record.asset_id,
            name: record.name,
            version: record.version,
            category: record.category,
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn delete_technology(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM technologies
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_technologies(
        &self,
        id_filter: Option<ID>,
        name: Option<String>,
        category: Option<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>> {
        let technologies = if let Some(id) = id_filter {
            // First, try to find technologies by a direct asset ID query
            let asset_records = sqlx::query!(
                r#"
                SELECT id, asset_id, name, version, category, created_at, updated_at
                FROM technologies
                WHERE asset_id = $1
                ORDER BY name
                LIMIT $2 OFFSET $3
                "#,
                id,
                limit as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await?;

            // If no records found by asset ID, try looking up by organization ID
            if asset_records.is_empty() {
                // Use a JOIN to find all technologies for assets within this organization
                let org_records = sqlx::query!(
                    r#"
                    SELECT t.id, t.asset_id, t.name, t.version, t.category, t.created_at, t.updated_at
                    FROM technologies t
                    JOIN assets a ON t.asset_id = a.id
                    WHERE a.organization_id = $1
                    ORDER BY t.name
                    LIMIT $2 OFFSET $3
                    "#,
                    id,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                org_records
                    .into_iter()
                    .map(|record| Technology {
                        id: record.id,
                        asset_id: record.asset_id,
                        name: record.name,
                        version: record.version,
                        category: record.category,
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            } else {
                // Return the records found by asset ID
                asset_records
                    .into_iter()
                    .map(|record| Technology {
                        id: record.id,
                        asset_id: record.asset_id,
                        name: record.name,
                        version: record.version,
                        category: record.category,
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
        } else if let Some(name_filter) = &name {
            if let Some(category_filter) = &category {
                // Both name and category filter
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, name, version, category, created_at, updated_at
                    FROM technologies
                    WHERE name ILIKE $1 AND category ILIKE $2
                    ORDER BY name
                    LIMIT $3 OFFSET $4
                    "#,
                    format!("%{}%", name_filter),
                    format!("%{}%", category_filter),
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Technology {
                        id: record.id,
                        asset_id: record.asset_id,
                        name: record.name,
                        version: record.version,
                        category: record.category,
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            } else {
                // Only name filter
                let records = sqlx::query!(
                    r#"
                    SELECT id, asset_id, name, version, category, created_at, updated_at
                    FROM technologies
                    WHERE name ILIKE $1
                    ORDER BY name
                    LIMIT $2 OFFSET $3
                    "#,
                    format!("%{}%", name_filter),
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;

                records
                    .into_iter()
                    .map(|record| Technology {
                        id: record.id,
                        asset_id: record.asset_id,
                        name: record.name,
                        version: record.version,
                        category: record.category,
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
        } else if let Some(category_filter) = &category {
            // Only category filter
            let records = sqlx::query!(
                r#"
                SELECT id, asset_id, name, version, category, created_at, updated_at
                FROM technologies
                WHERE category ILIKE $1
                ORDER BY name
                LIMIT $2 OFFSET $3
                "#,
                format!("%{}%", category_filter),
                limit as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await?;

            records
                .into_iter()
                .map(|record| Technology {
                    id: record.id,
                    asset_id: record.asset_id,
                    name: record.name,
                    version: record.version,
                    category: record.category,
                    created_at: from_offset_datetime(Some(record.created_at)),
                    updated_at: from_offset_datetime(Some(record.updated_at)),
                })
                .collect()
        } else {
            // No filters
            let records = sqlx::query!(
                r#"
                SELECT id, asset_id, name, version, category, created_at, updated_at
                FROM technologies
                ORDER BY name
                LIMIT $1 OFFSET $2
                "#,
                limit as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await?;

            records
                .into_iter()
                .map(|record| Technology {
                    id: record.id,
                    asset_id: record.asset_id,
                    name: record.name,
                    version: record.version,
                    category: record.category,
                    created_at: from_offset_datetime(Some(record.created_at)),
                    updated_at: from_offset_datetime(Some(record.updated_at)),
                })
                .collect()
        };

        Ok(technologies)
    }

    async fn count_technologies(
        &self,
        asset_id: Option<ID>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<usize> {
        let count = if let Some(id) = asset_id {
            // We have an asset ID filter
            if let (Some(name_filter), Some(category_filter)) = (&name, &category) {
                // Asset ID + name + category filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE asset_id = $1 AND name ILIKE $2 AND category ILIKE $3
                    "#,
                    id,
                    format!("%{}%", name_filter),
                    format!("%{}%", category_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else if let Some(name_filter) = &name {
                // Asset ID + name filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE asset_id = $1 AND name ILIKE $2
                    "#,
                    id,
                    format!("%{}%", name_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else if let Some(category_filter) = &category {
                // Asset ID + category filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE asset_id = $1 AND category ILIKE $2
                    "#,
                    id,
                    format!("%{}%", category_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else {
                // Only asset ID filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE asset_id = $1
                    "#,
                    id
                )
                .fetch_one(&self.pool)
                .await?
            }
        } else {
            // No asset ID filter
            if let (Some(name_filter), Some(category_filter)) = (&name, &category) {
                // Name + category filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE name ILIKE $1 AND category ILIKE $2
                    "#,
                    format!("%{}%", name_filter),
                    format!("%{}%", category_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else if let Some(name_filter) = &name {
                // Only name filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE name ILIKE $1
                    "#,
                    format!("%{}%", name_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else if let Some(category_filter) = &category {
                // Only category filter
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    WHERE category ILIKE $1
                    "#,
                    format!("%{}%", category_filter)
                )
                .fetch_one(&self.pool)
                .await?
            } else {
                // No filters
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM technologies
                    "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(count.unwrap_or(0) as usize)
    }
}
