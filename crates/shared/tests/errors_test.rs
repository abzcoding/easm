#[cfg(test)]
mod tests {
    use shared::errors::{AppError, Result};
    use std::io;

    #[test]
    fn test_error_constructors() {
        let db_error = AppError::database("Database connection failed");
        let validation_error = AppError::validation("Invalid input");
        let auth_error = AppError::authentication("Invalid credentials");
        let authz_error = AppError::authorization("Insufficient permissions");
        let not_found_error = AppError::not_found("Resource not found");
        let ext_service_error = AppError::external_service("External API failed");
        let config_error = AppError::configuration("Invalid configuration");
        let internal_error = AppError::internal("Unknown error");

        // Since the actual enum variants contain complex types, let's test using error display format
        assert!(format!("{}", db_error).contains("Database error"));
        assert!(format!("{}", validation_error).contains("Validation error"));
        assert!(format!("{}", auth_error).contains("Authentication error"));
        assert!(format!("{}", authz_error).contains("Permission denied"));
        assert!(format!("{}", not_found_error).contains("Not found"));
        assert!(format!("{}", ext_service_error).contains("External service error"));
        assert!(format!("{}", config_error).contains("Configuration error"));
        assert!(format!("{}", internal_error).contains("Internal server error"));
    }

    #[test]
    fn test_error_display() {
        let db_error = AppError::database("Database connection failed");
        assert!(format!("{}", db_error).contains("Database error"));

        let validation_error = AppError::validation("Invalid input");
        assert!(format!("{}", validation_error).contains("Validation error"));
    }

    #[test]
    fn test_error_from_sqlx() {
        // We can't easily create a real sqlx::Error, so we'll just test the From impl logic
        let sqlx_err = sqlx::Error::RowNotFound;
        let error = AppError::from(sqlx_err);

        assert!(matches!(error, AppError::Database(_)));
        assert!(format!("{}", error).contains("Database error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = AppError::from(io_err);

        assert!(matches!(error, AppError::Internal(_)));
        assert!(format!("{}", error).contains("IO error"));
        assert!(format!("{}", error).contains("file not found"));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
        let error = AppError::from(json_err);

        assert!(matches!(error, AppError::Internal(_)));
        assert!(format!("{}", error).contains("JSON error"));
    }

    #[test]
    fn test_result_type() {
        // Test that our Result type alias works as expected
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(AppError::not_found("Test"));

        assert_eq!(success.unwrap(), 42);
        assert!(failure.is_err());
    }
}
