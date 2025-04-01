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
        asset_id: Option<ID>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Technology>> {
        let technologies = match asset_id {
            Some(id) => {
                let records = sqlx::query!(
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
            None => {
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
            }
        };

        Ok(technologies)
    }

    async fn count_technologies(&self, asset_id: Option<ID>) -> Result<usize> {
        let count = match asset_id {
            Some(id) => {
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
            None => {
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
