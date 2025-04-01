use crate::utils::{
    from_offset_datetime, from_option_offset_datetime, to_offset_datetime,
    to_option_offset_datetime,
};
use async_trait::async_trait;
use backend::{
    models::{Asset, DiscoveryJob, JobAssetLink},
    traits::DiscoveryJobRepository,
    Result,
};
use shared::types::{AssetStatus, AssetType, JobStatus, JobType, ID};
use sqlx::PgPool;

/// PostgreSQL implementation of the DiscoveryJob Repository
pub struct PgDiscoveryJobRepository {
    pool: PgPool,
}

impl PgDiscoveryJobRepository {
    /// Create a new PgDiscoveryJobRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DiscoveryJobRepository for PgDiscoveryJobRepository {
    async fn create_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob> {
        // Convert DateTime types for database operation
        let started_at = to_option_offset_datetime(job.started_at);
        let completed_at = to_option_offset_datetime(job.completed_at);
        let created_at = to_offset_datetime(job.created_at);
        let updated_at = to_offset_datetime(job.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO discovery_jobs (
                id, organization_id, job_type, status, target, 
                started_at, completed_at, logs, configuration, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING 
                id, organization_id, job_type as "job_type: JobType", status as "status: JobStatus", 
                target, started_at, completed_at, logs, configuration, created_at, updated_at
            "#,
            job.id,
            job.organization_id,
            job.job_type as JobType,
            job.status as JobStatus,
            job.target,
            started_at,
            completed_at,
            job.logs,
            job.configuration,
            created_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(DiscoveryJob {
            id: record.id,
            organization_id: record.organization_id,
            job_type: record.job_type,
            status: record.status,
            target: record.target,
            started_at: from_option_offset_datetime(record.started_at),
            completed_at: from_option_offset_datetime(record.completed_at),
            logs: record.logs,
            configuration: record
                .configuration
                .expect("Job configuration should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn get_job(&self, id: ID) -> Result<DiscoveryJob> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, organization_id, job_type as "job_type: JobType", status as "status: JobStatus", 
                target, started_at, completed_at, logs, configuration, created_at, updated_at
            FROM discovery_jobs
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(DiscoveryJob {
            id: record.id,
            organization_id: record.organization_id,
            job_type: record.job_type,
            status: record.status,
            target: record.target,
            started_at: from_option_offset_datetime(record.started_at),
            completed_at: from_option_offset_datetime(record.completed_at),
            logs: record.logs,
            configuration: record
                .configuration
                .expect("Job configuration should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn update_job(&self, job: &DiscoveryJob) -> Result<DiscoveryJob> {
        // Convert DateTime types for database operation
        let started_at = to_option_offset_datetime(job.started_at);
        let completed_at = to_option_offset_datetime(job.completed_at);
        let updated_at = to_offset_datetime(job.updated_at);

        let record = sqlx::query!(
            r#"
            UPDATE discovery_jobs
            SET 
                organization_id = $2, job_type = $3, status = $4, target = $5,
                started_at = $6, completed_at = $7, logs = $8, configuration = $9, updated_at = $10
            WHERE id = $1
            RETURNING 
                id, organization_id, job_type as "job_type: JobType", status as "status: JobStatus", 
                target, started_at, completed_at, logs, configuration, created_at, updated_at
            "#,
            job.id,
            job.organization_id,
            job.job_type as JobType,
            job.status as JobStatus,
            job.target,
            started_at,
            completed_at,
            job.logs,
            job.configuration,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(DiscoveryJob {
            id: record.id,
            organization_id: record.organization_id,
            job_type: record.job_type,
            status: record.status,
            target: record.target,
            started_at: from_option_offset_datetime(record.started_at),
            completed_at: from_option_offset_datetime(record.completed_at),
            logs: record.logs,
            configuration: record
                .configuration
                .expect("Job configuration should not be null"),
            created_at: from_offset_datetime(Some(record.created_at)),
            updated_at: from_offset_datetime(Some(record.updated_at)),
        })
    }

    async fn delete_job(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM discovery_jobs
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_jobs(
        &self,
        organization_id: Option<ID>,
        _job_type: Option<JobType>,
        _status: Option<JobStatus>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DiscoveryJob>> {
        // Simplified implementation that only handles organization_id filter
        let jobs = match organization_id {
            Some(org_id) => {
                let records = sqlx::query!(
                    r#"
                    SELECT
                        id, organization_id, job_type as "job_type: JobType", status as "status: JobStatus", 
                        target, started_at, completed_at, logs, configuration, created_at, updated_at
                    FROM discovery_jobs
                    WHERE organization_id = $1
                    ORDER BY created_at DESC
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
                    .map(|record| DiscoveryJob {
                        id: record.id,
                        organization_id: record.organization_id,
                        job_type: record.job_type,
                        status: record.status,
                        target: record.target,
                        started_at: from_option_offset_datetime(record.started_at),
                        completed_at: from_option_offset_datetime(record.completed_at),
                        logs: record.logs,
                        configuration: record
                            .configuration
                            .expect("Job configuration should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
            None => {
                let records = sqlx::query!(
                    r#"
                    SELECT
                        id, organization_id, job_type as "job_type: JobType", status as "status: JobStatus", 
                        target, started_at, completed_at, logs, configuration, created_at, updated_at
                    FROM discovery_jobs
                    ORDER BY created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                records
                    .into_iter()
                    .map(|record| DiscoveryJob {
                        id: record.id,
                        organization_id: record.organization_id,
                        job_type: record.job_type,
                        status: record.status,
                        target: record.target,
                        started_at: from_option_offset_datetime(record.started_at),
                        completed_at: from_option_offset_datetime(record.completed_at),
                        logs: record.logs,
                        configuration: record
                            .configuration
                            .expect("Job configuration should not be null"),
                        created_at: from_offset_datetime(Some(record.created_at)),
                        updated_at: from_offset_datetime(Some(record.updated_at)),
                    })
                    .collect()
            }
        };

        Ok(jobs)
    }

    async fn count_jobs(
        &self,
        organization_id: Option<ID>,
        _job_type: Option<JobType>,
        _status: Option<JobStatus>,
    ) -> Result<usize> {
        let count = match organization_id {
            Some(org_id) => {
                sqlx::query_scalar!(
                    r#"
                SELECT COUNT(*) as count
                FROM discovery_jobs
                WHERE organization_id = $1
                "#,
                    org_id
                )
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_scalar!(
                    r#"
                SELECT COUNT(*) as count
                FROM discovery_jobs
                "#
                )
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(count.unwrap_or(0) as usize)
    }

    async fn create_job_asset_link(&self, link: &JobAssetLink) -> Result<JobAssetLink> {
        sqlx::query!(
            r#"
            INSERT INTO job_asset_links (job_id, asset_id)
            VALUES ($1, $2)
            ON CONFLICT (job_id, asset_id) DO NOTHING
            "#,
            link.job_id,
            link.asset_id
        )
        .execute(&self.pool)
        .await?;

        Ok(link.clone())
    }

    async fn get_job_assets(&self, job_id: ID) -> Result<Vec<Asset>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                a.id, a.organization_id, a.asset_type as "asset_type: AssetType", 
                a.value, a.status as "status: AssetStatus", a.first_seen, 
                a.last_seen, a.created_at, a.updated_at, a.attributes
            FROM assets a
            JOIN job_asset_links j ON a.id = j.asset_id
            WHERE j.job_id = $1
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Convert from DB types to model types
        let assets = records
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
            .collect();

        Ok(assets)
    }
}
