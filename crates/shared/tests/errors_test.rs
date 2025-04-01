#[cfg(test)]
mod tests {
    use shared::errors::{Error, Result};
    use std::io;

    #[test]
    fn test_error_constructors() {
        let db_error = Error::database("Database connection failed");
        let validation_error = Error::validation("Invalid input");
        let auth_error = Error::authentication("Invalid credentials");
        let authz_error = Error::authorization("Insufficient permissions");
        let not_found_error = Error::not_found("Resource not found");
        let ext_service_error = Error::external_service("External API failed");
        let config_error = Error::configuration("Invalid configuration");
        let internal_error = Error::internal("Unknown error");

        match db_error {
            Error::Database(msg) => assert_eq!(msg, "Database connection failed"),
            _ => panic!("Wrong error variant"),
        }

        match validation_error {
            Error::Validation(msg) => assert_eq!(msg, "Invalid input"),
            _ => panic!("Wrong error variant"),
        }

        match auth_error {
            Error::Authentication(msg) => assert_eq!(msg, "Invalid credentials"),
            _ => panic!("Wrong error variant"),
        }

        match authz_error {
            Error::Authorization(msg) => assert_eq!(msg, "Insufficient permissions"),
            _ => panic!("Wrong error variant"),
        }

        match not_found_error {
            Error::NotFound(msg) => assert_eq!(msg, "Resource not found"),
            _ => panic!("Wrong error variant"),
        }

        match ext_service_error {
            Error::ExternalService(msg) => assert_eq!(msg, "External API failed"),
            _ => panic!("Wrong error variant"),
        }

        match config_error {
            Error::Configuration(msg) => assert_eq!(msg, "Invalid configuration"),
            _ => panic!("Wrong error variant"),
        }

        match internal_error {
            Error::Internal(msg) => assert_eq!(msg, "Unknown error"),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let db_error = Error::database("Database connection failed");
        assert_eq!(
            format!("{}", db_error),
            "Database error: Database connection failed"
        );

        let validation_error = Error::validation("Invalid input");
        assert_eq!(
            format!("{}", validation_error),
            "Validation error: Invalid input"
        );
    }

    #[test]
    fn test_error_from_sqlx() {
        // We can't easily create a real sqlx::Error, so we'll just test the From impl logic
        let sqlx_err = sqlx::Error::RowNotFound;
        let error = Error::from(sqlx_err);

        match error {
            Error::Database(msg) => assert_eq!(msg, "row not found"),
            _ => panic!("Expected Database error variant"),
        }
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = Error::from(io_err);

        match error {
            Error::Internal(msg) => {
                assert!(msg.starts_with("IO error: "));
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected Internal error variant"),
        }
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
        let error = Error::from(json_err);

        match error {
            Error::Internal(msg) => {
                assert!(msg.starts_with("JSON error: "));
            }
            _ => panic!("Expected Internal error variant"),
        }
    }

    #[test]
    fn test_result_type() {
        // Test that our Result type alias works as expected
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(Error::not_found("Test"));

        assert_eq!(success.unwrap(), 42);
        assert!(failure.is_err());
    }
}
