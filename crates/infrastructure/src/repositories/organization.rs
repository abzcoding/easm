use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{models::Organization, traits::OrganizationRepository, Result};
use shared::types::ID;
use sqlx::PgPool;

/// PostgreSQL implementation of the Organization Repository
pub struct PgOrganizationRepository {
    pool: PgPool,
}

impl PgOrganizationRepository {
    /// Create a new PgOrganizationRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    async fn create_organization(&self, organization: &Organization) -> Result<Organization> {
        let created_at = to_offset_datetime(organization.created_at);
        let updated_at = to_offset_datetime(organization.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, created_at, updated_at
            "#,
            organization.id,
            organization.name,
            created_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Organization {
            id: record.id,
            name: record.name,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn get_organization(&self, id: ID) -> Result<Organization> {
        let record = sqlx::query!(
            r#"
            SELECT id, name, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Organization {
            id: record.id,
            name: record.name,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn update_organization(&self, organization: &Organization) -> Result<Organization> {
        let updated_at = to_offset_datetime(organization.updated_at);

        let record = sqlx::query!(
            r#"
            UPDATE organizations
            SET name = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, name, created_at, updated_at
            "#,
            organization.id,
            organization.name,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Organization {
            id: record.id,
            name: record.name,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn delete_organization(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM organizations
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_organizations(&self, limit: usize, offset: usize) -> Result<Vec<Organization>> {
        let records = sqlx::query!(
            r#"
            SELECT id, name, created_at, updated_at
            FROM organizations
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let organizations = records
            .into_iter()
            .map(|record| Organization {
                id: record.id,
                name: record.name,
                created_at: from_offset_datetime(Some(
                    record.created_at.expect("created_at should not be null"),
                )),
                updated_at: from_offset_datetime(Some(
                    record.updated_at.expect("updated_at should not be null"),
                )),
            })
            .collect();

        Ok(organizations)
    }

    async fn count_organizations(&self) -> Result<usize> {
        let record = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM organizations
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.count.unwrap_or(0) as usize)
    }
}
