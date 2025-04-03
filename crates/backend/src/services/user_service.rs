use std::sync::Arc;
use argon2::{
    self,
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use crate::{
    errors::Error as BackendError,
    models::User,
    traits::UserRepository,
};
use crate::errors::Result;
use uuid;


pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn register_user(
        &self,
        organization_id: &uuid::Uuid,
        email: &str,
        password: &str,
    ) -> Result<User> {
        // TODO: Validate email uniqueness
        // let existing_user = self.user_repository.find_by_email(email).await?;
        // if existing_user.is_some() {
        //     return Err(BackendError::Conflict("Email already exists".to_string()));
        // }

        // Hash password
        let password_hash = hash_password(password)?;

        // Create user object
        let username = email.split('@').next().unwrap_or(email).to_string();

        let user = User::new(
            *organization_id,
            username,
            email.to_string(),
            password_hash,
            Some("user".parse().map_err(|_| BackendError::Internal("Invalid default role".to_string()))?),
        );

        // Save user
        let created_user = self.user_repository.create_user(&user).await?;
        Ok(created_user)
    }

    pub async fn login_user(&self, email: &str, password: &str) -> Result<User> {
        // Find user by email
        let user = self
            .user_repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| BackendError::NotFound("User not found".to_string()))?;

        // Verify password
        if verify_password(&user.password_hash, password)? {
            Ok(user)
        } else {
            Err(BackendError::Authentication("Invalid credentials".to_string()))
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = self.user_repository.find_by_email(email).await?;
        Ok(user)
    }
}

// Helper function for password hashing
fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    // Argon2::default() returns the recommended Argon2id variant.
    let argon2 = Argon2::default();

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
    
    let argon2 = Argon2::default(); // Use the same variant for verification

    // Verify password against hash
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true), // Password matches
        Err(argon2::password_hash::Error::Password) => Ok(false), // Password does not match
        Err(e) => Err(BackendError::Internal(format!(
            "Password verification failed unexpectedly: {}",
            e
        ))), // Other errors (e.g., invalid hash format)
    }
}

// Helper function to generate salt - No longer needed with PasswordHasher
// fn generate_salt() -> Result<Vec<u8>> { ... } 