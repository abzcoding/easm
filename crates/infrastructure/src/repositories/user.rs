use crate::utils::{from_offset_datetime, to_offset_datetime};
use async_trait::async_trait;
use backend::{errors::Error as BackendError, models::User, traits::UserRepository, Result};
use shared::types::{UserRole, ID};
use sqlx::{PgPool, Row};

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

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        // Convert record to Option<User>
        record
            .map(|r| User {
                id: r.id,
                organization_id: r
                    .organization_id
                    .expect("organization_id should not be null"),
                username: r.username,
                email: r.email,
                role: r.role.expect("role should not be null"),
                password_hash: r.password_hash,
                created_at: from_offset_datetime(Some(
                    r.created_at.expect("created_at should not be null"),
                )),
                updated_at: from_offset_datetime(Some(
                    r.updated_at.expect("updated_at should not be null"),
                )),
            })
            .map_or(Ok(None), |u| Ok(Some(u)))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        // Convert record to Option<User>
        record
            .map(|r| User {
                id: r.id,
                organization_id: r
                    .organization_id
                    .expect("organization_id should not be null"),
                username: r.username,
                email: r.email,
                role: r.role.expect("role should not be null"),
                password_hash: r.password_hash,
                created_at: from_offset_datetime(Some(
                    r.created_at.expect("created_at should not be null"),
                )),
                updated_at: from_offset_datetime(Some(
                    r.updated_at.expect("updated_at should not be null"),
                )),
            })
            .map_or(Ok(None), |u| Ok(Some(u)))
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
        // Use QueryBuilder to dynamically build the query
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, organization_id, username, email, role, password_hash, created_at, updated_at FROM users WHERE 1=1"
        );

        // Add filters conditionally
        if let Some(org_id) = organization_id {
            query_builder.push(" AND organization_id = ");
            query_builder.push_bind(org_id);
        }

        if let Some(role_val) = role {
            query_builder.push(" AND role = ");
            query_builder.push_bind(role_val);
        }

        // Add ordering and pagination
        query_builder
            .push(" ORDER BY username LIMIT ")
            .push_bind(limit as i64)
            .push(" OFFSET ")
            .push_bind(offset as i64);

        // Build and execute the query
        let query = query_builder.build_query_as::<(
            ID,
            Option<ID>,
            String,
            String,
            UserRole,
            String,
            Option<sqlx::types::time::OffsetDateTime>,
            Option<sqlx::types::time::OffsetDateTime>,
        )>();

        let records = query.fetch_all(&self.pool).await?;

        // Map the results to User objects
        let users = records
            .into_iter()
            .map(|record| {
                let (
                    id,
                    organization_id,
                    username,
                    email,
                    role,
                    password_hash,
                    created_at,
                    updated_at,
                ) = record;
                User {
                    id,
                    organization_id: organization_id.expect("organization_id should not be null"),
                    username,
                    email,
                    role,
                    password_hash,
                    created_at: from_offset_datetime(created_at),
                    updated_at: from_offset_datetime(updated_at),
                }
            })
            .collect();

        Ok(users)
    }

    async fn count_users(
        &self,
        organization_id: Option<ID>,
        role: Option<UserRole>,
    ) -> Result<usize> {
        // Use QueryBuilder to dynamically build the count query
        let mut query_builder =
            sqlx::QueryBuilder::new("SELECT COUNT(*) as count FROM users WHERE 1=1");

        // Add filters conditionally
        if let Some(org_id) = organization_id {
            query_builder.push(" AND organization_id = ");
            query_builder.push_bind(org_id);
        }

        if let Some(role_val) = role {
            query_builder.push(" AND role = ");
            query_builder.push_bind(role_val);
        }

        // Build and execute the query
        let query = query_builder.build();
        let row = query.fetch_one(&self.pool).await?;

        // Get count value with error handling
        let count: i64 = match row.try_get("count") {
            Ok(val) => val,
            Err(_) => 0,
        };

        Ok(count as usize)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let record = sqlx::query!(
            r#"
            SELECT id, organization_id, username, email, role as "role: UserRole", password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| User {
            id: r.id,
            organization_id: r
                .organization_id
                .expect("organization_id should not be null"),
            username: r.username,
            email: r.email,
            role: r.role.expect("role should not be null"),
            password_hash: r.password_hash,
            created_at: from_offset_datetime(Some(
                r.created_at.expect("created_at should not be null"),
            )),
            updated_at: from_offset_datetime(Some(
                r.updated_at.expect("updated_at should not be null"),
            )),
        }))
    }

    async fn atomic_register_user(&self, email: &str, user: &User) -> Result<User> {
        // Begin a transaction
        let mut tx = self.pool.begin().await?;

        // First check if the email already exists within the transaction
        let record = sqlx::query!(
            r#"
            SELECT id FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&mut *tx)
        .await?;

        // If email exists, abort with conflict error
        if record.is_some() {
            tx.rollback().await?;
            return Err(BackendError::Conflict("Email already exists".to_string()));
        }

        // Email doesn't exist, create the user
        // Convert DateTime types for database operation
        let created_at = to_offset_datetime(user.created_at);
        let updated_at = to_offset_datetime(user.updated_at);

        // Insert the user
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
        .fetch_one(&mut *tx)
        .await?;

        // Commit the transaction
        tx.commit().await?;

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
}
