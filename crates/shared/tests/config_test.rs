#[cfg(test)]
mod tests {
    use shared::config::{Config, ConfigError, Environment};
    use std::env;

    #[test]
    fn test_environment_enum() {
        assert_eq!(Environment::Development, Environment::Development);
        assert_ne!(Environment::Development, Environment::Production);
        assert_ne!(Environment::Development, Environment::Test);
    }

    #[test]
    fn test_config_error() {
        let missing_err = ConfigError::MissingEnv("TEST_VAR");
        let invalid_err = ConfigError::InvalidValue("TEST_VAR");

        assert_ne!(missing_err, invalid_err);
        assert_eq!(
            format!("{}", missing_err),
            "Missing environment variable: TEST_VAR"
        );
        assert_eq!(
            format!("{}", invalid_err),
            "Invalid value for environment variable: TEST_VAR"
        );
    }

    #[test]
    fn test_config_environment_methods() {
        let dev_config = Config {
            database_url: "test".into(),
            redis_url: None,
            host: "127.0.0.1".parse().unwrap(),
            port: 3000,
            jwt_secret: "secret".into(),
            jwt_expiration: 86400,
            environment: Environment::Development,
            log_level: "info".into(),
            max_concurrent_tasks: 10,
        };

        let prod_config = Config {
            database_url: "test".into(),
            redis_url: None,
            host: "127.0.0.1".parse().unwrap(),
            port: 3000,
            jwt_secret: "secret".into(),
            jwt_expiration: 86400,
            environment: Environment::Production,
            log_level: "info".into(),
            max_concurrent_tasks: 10,
        };

        let test_config = Config {
            database_url: "test".into(),
            redis_url: None,
            host: "127.0.0.1".parse().unwrap(),
            port: 3000,
            jwt_secret: "secret".into(),
            jwt_expiration: 86400,
            environment: Environment::Test,
            log_level: "info".into(),
            max_concurrent_tasks: 10,
        };

        assert!(dev_config.is_development());
        assert!(!dev_config.is_production());
        assert!(!dev_config.is_test());

        assert!(!prod_config.is_development());
        assert!(prod_config.is_production());
        assert!(!prod_config.is_test());

        assert!(!test_config.is_development());
        assert!(!test_config.is_production());
        assert!(test_config.is_test());
    }

    #[test]
    fn test_config_from_env_with_defaults() {
        // Clear any existing env vars that might interfere
        env::remove_var("HOST");
        env::remove_var("PORT");
        env::remove_var("JWT_EXPIRATION");
        env::remove_var("ENVIRONMENT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("MAX_CONCURRENT_TASKS");

        // Set required env vars
        env::set_var("DATABASE_URL", "postgres://test");
        env::set_var("JWT_SECRET", "test_secret");

        // Test with defaults
        let config = Config::from_env().unwrap();

        assert_eq!(config.database_url, "postgres://test");
        assert_eq!(config.host.to_string(), "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.jwt_secret, "test_secret");
        assert_eq!(config.jwt_expiration, 86400);
        assert!(config.is_development());
        assert_eq!(config.log_level, "info");
        assert_eq!(config.max_concurrent_tasks, 10);

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
    }
}
