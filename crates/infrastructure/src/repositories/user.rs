use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{models::User, traits::UserRepository, Result};
use shared::types::{UserRole, ID};
use sqlx::PgPool;

/// PostgreSQL implementation of the User Repository
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Create a new PgUserRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(&self, user: &User) -> Result<User> {
        // Convert DateTime types for database operation
        let created_at = to_offset_datetime(user.created_at);
        let updated_at = to_offset_datetime(user.updated_at);

        let record = sqlx::query!(
            r#"
            INSERT INTO users (id, organization_id, username, email, role, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            "#,
            user.id,
            user.organization_id,
            user.username,
            user.email,
            user.role as UserRole,
            user.password_hash,
            created_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(User {
            id: record.id,
            organization_id: record
                .organization_id
                .expect("organization_id should not be null"),
            username: record.username,
            email: record.email,
            role: record.role.expect("role should not be null"),
            password_hash: record.password_hash,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn get_user(&self, id: ID) -> Result<User> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(User {
            id: record.id,
            organization_id: record
                .organization_id
                .expect("organization_id should not be null"),
            username: record.username,
            email: record.email,
            role: record.role.expect("role should not be null"),
            password_hash: record.password_hash,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(User {
            id: record.id,
            organization_id: record
                .organization_id
                .expect("organization_id should not be null"),
            username: record.username,
            email: record.email,
            role: record.role.expect("role should not be null"),
            password_hash: record.password_hash,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(User {
            id: record.id,
            organization_id: record
                .organization_id
                .expect("organization_id should not be null"),
            username: record.username,
            email: record.email,
            role: record.role.expect("role should not be null"),
            password_hash: record.password_hash,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn update_user(&self, user: &User) -> Result<User> {
        // Convert DateTime types for database operation
        let updated_at = to_offset_datetime(user.updated_at);

        let record = sqlx::query!(
            r#"
            UPDATE users
            SET organization_id = $2, username = $3, email = $4, role = $5, password_hash = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            "#,
            user.id,
            user.organization_id,
            user.username,
            user.email,
            user.role as UserRole,
            user.password_hash,
            updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Convert back from DB types to model types
        Ok(User {
            id: record.id,
            organization_id: record
                .organization_id
                .expect("organization_id should not be null"),
            username: record.username,
            email: record.email,
            role: record.role.expect("role should not be null"),
            password_hash: record.password_hash,
            created_at: from_offset_datetime(Some(
                record.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                record.updated_at.expect("updated_at should not be null"),
            )),
        })
    }

    async fn delete_user(&self, id: ID) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_users(
        &self,
        organization_id: Option<ID>,
        role: Option<UserRole>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<User>> {
        if let Some(org_id) = organization_id {
            if let Some(role_val) = role {
                // Both organization_id and role are provided
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
                    FROM users
                    WHERE organization_id = $1 AND role = $2
                    ORDER BY username
                    LIMIT $3 OFFSET $4
                    "#,
                    org_id,
                    role_val as UserRole,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                let users = records
                    .into_iter()
                    .map(|record| User {
                        id: record.id,
                        organization_id: record
                            .organization_id
                            .expect("organization_id should not be null"),
                        username: record.username,
                        email: record.email,
                        role: record.role.expect("role should not be null"),
                        password_hash: record.password_hash,
                        created_at: from_offset_datetime(Some(
                            record.created_at.expect("created_at should not be null"),
                        )),
                        updated_at: from_offset_datetime(Some(
                            record.updated_at.expect("updated_at should not be null"),
                        )),
                    })
                    .collect();
                return Ok(users);
            } else {
                // Only organization_id is provided
                let records = sqlx::query!(
                    r#"
                    SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
                    FROM users
                    WHERE organization_id = $1
                    ORDER BY username
                    LIMIT $2 OFFSET $3
                    "#,
                    org_id,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?;
                let users = records
                    .into_iter()
                    .map(|record| User {
                        id: record.id,
                        organization_id: record
                            .organization_id
                            .expect("organization_id should not be null"),
                        username: record.username,
                        email: record.email,
                        role: record.role.expect("role should not be null"),
                        password_hash: record.password_hash,
                        created_at: from_offset_datetime(Some(
                            record.created_at.expect("created_at should not be null"),
                        )),
                        updated_at: from_offset_datetime(Some(
                            record.updated_at.expect("updated_at should not be null"),
                        )),
                    })
                    .collect();
                return Ok(users);
            }
        } else if let Some(role_val) = role {
            // Only role is provided
            let records = sqlx::query!(
                r#"
                SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
                FROM users
                WHERE role = $1
                ORDER BY username
                LIMIT $2 OFFSET $3
                "#,
                role_val as UserRole,
                limit as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await?;
            let users = records
                .into_iter()
                .map(|record| User {
                    id: record.id,
                    organization_id: record
                        .organization_id
                        .expect("organization_id should not be null"),
                    username: record.username,
                    email: record.email,
                    role: record.role.expect("role should not be null"),
                    password_hash: record.password_hash,
                    created_at: from_offset_datetime(Some(
                        record.created_at.expect("created_at should not be null"),
                    )),
                    updated_at: from_offset_datetime(Some(
                        record.updated_at.expect("updated_at should not be null"),
                    )),
                })
                .collect();
            return Ok(users);
        } else {
            // No filters provided
            let records = sqlx::query!(
                r#"
                SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
                FROM users
                ORDER BY username
                LIMIT $1 OFFSET $2
                "#,
                limit as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await?;
            let users = records
                .into_iter()
                .map(|record| User {
                    id: record.id,
                    organization_id: record
                        .organization_id
                        .expect("organization_id should not be null"),
                    username: record.username,
                    email: record.email,
                    role: record.role.expect("role should not be null"),
                    password_hash: record.password_hash,
                    created_at: from_offset_datetime(Some(
                        record.created_at.expect("created_at should not be null"),
                    )),
                    updated_at: from_offset_datetime(Some(
                        record.updated_at.expect("updated_at should not be null"),
                    )),
                })
                .collect();
            return Ok(users);
        }
    }

    async fn count_users(
        &self,
        organization_id: Option<ID>,
        role: Option<UserRole>,
    ) -> Result<usize> {
        let count = if let Some(org_id) = organization_id {
            if let Some(role_val) = role {
                // Both organization_id and role are provided
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM users
                    WHERE organization_id = $1 AND role = $2
                    "#,
                    org_id,
                    role_val as UserRole,
                )
                .fetch_one(&self.pool)
                .await?
            } else {
                // Only organization_id is provided
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) as count
                    FROM users
                    WHERE organization_id = $1
                    "#,
                    org_id,
                )
                .fetch_one(&self.pool)
                .await?
            }
        } else if let Some(role_val) = role {
            // Only role is provided
            sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) as count
                FROM users
                WHERE role = $1
                "#,
                role_val as UserRole,
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            // No filters provided
            sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) as count
                FROM users
                "#
            )
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count.unwrap_or(0) as usize)
    }
}
