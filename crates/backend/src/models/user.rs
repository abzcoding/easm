use serde::{Deserialize, Serialize};
use shared::types::{Timestamp, UserRole, ID};

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    pub id: ID,

    /// Organization this user belongs to
    pub organization_id: ID,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// User role
    pub role: UserRole,

    /// Password hash (never exposed in API responses)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Creation timestamp
    pub created_at: Timestamp,

    /// Last updated timestamp
    pub updated_at: Timestamp,
}

impl User {
    /// Create a new user with default role (Analyst)
    pub fn new(
        organization_id: ID,
        username: String,
        email: String,
        password_hash: String,
        role: Option<UserRole>,
    ) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            username,
            email,
            role: role.unwrap_or(UserRole::Analyst),
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}
