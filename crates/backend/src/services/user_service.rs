use crate::errors::Result;
use crate::{
    errors::Error as BackendError,
    models::User,
    traits::{OrganizationRepository, UserRepository, UserService},
};
use argon2::{
    self,
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use std::sync::Arc;
use uuid;

pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
    org_repository: Arc<dyn OrganizationRepository>,
}

impl UserServiceImpl {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        org_repository: Arc<dyn OrganizationRepository>,
    ) -> Self {
        Self {
            repository,
            org_repository,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(
        &self,
        organization_id: &uuid::Uuid,
        email: &str,
        password: &str,
    ) -> Result<User> {
        // Check if organization exists
        self.org_repository
            .get_organization(*organization_id)
            .await?;

        // Hash password
        let password_hash = hash_password(password)?;

        // Create user object
        let username = email.split('@').next().unwrap_or(email).to_string();

        let user = User::new(
            *organization_id,
            username,
            email.to_string(),
            password_hash,
            Some(
                "analyst"
                    .parse()
                    .map_err(|_| BackendError::Internal("Invalid default role".to_string()))?,
            ),
        );

        // Use atomic operation to check email and create user
        let created_user = self.repository.atomic_register_user(email, &user).await?;

        Ok(created_user)
    }

    async fn login_user(&self, email: &str, password: &str) -> Result<User> {
        // Find user by email
        let user = self
            .repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| BackendError::NotFound("User not found".to_string()))?;

        // Verify password
        if verify_password(&user.password_hash, password)? {
            Ok(user)
        } else {
            Err(BackendError::Authentication(
                "Invalid credentials".to_string(),
            ))
        }
    }
}

// Helper function for password hashing
fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    // Configure Argon2 with explicit parameters for better security
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id, // Argon2id variant (more secure)
        argon2::Version::V0x13,      // Latest version
        argon2::Params::new(
            65536,    // Memory cost (64MB)
            2,        // Iterations
            1,        // Parallelism
            Some(32), // Output size (32 bytes)
        )
        .map_err(|e| BackendError::Internal(format!("Invalid Argon2 parameters: {}", e)))?,
    );

    // Hash password to PHC string ($argon2id$v=19$...) format
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| BackendError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();

    Ok(password_hash)
}

// Helper function to verify password
fn verify_password(hash: &str, password: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| BackendError::Internal(format!("Invalid password hash format: {}", e)))?;

    // Configure Argon2 with the same parameters used for hashing
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id, // Same algorithm
        argon2::Version::V0x13,      // Same version
        argon2::Params::new(
            65536,    // Same memory cost
            2,        // Same iterations
            1,        // Same parallelism
            Some(32), // Same output size
        )
        .map_err(|e| BackendError::Internal(format!("Invalid Argon2 parameters: {}", e)))?,
    );

    // Verify password against hash
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),                                        // Password matches
        Err(argon2::password_hash::Error::Password) => Ok(false), // Password does not match
        Err(e) => Err(BackendError::Internal(format!(
            "Password verification failed unexpectedly: {}",
            e
        ))), // Other errors (e.g., invalid hash format)
    }
}

// Helper function to generate salt - No longer needed with PasswordHasher
// fn generate_salt() -> Result<Vec<u8>> { ... }
