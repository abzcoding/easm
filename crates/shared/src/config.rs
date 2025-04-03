use dotenvy::dotenv;
use serde::Deserialize;
use std::env;
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub host: IpAddr,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub environment: Environment,
    pub log_level: String,
    pub max_concurrent_tasks: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenv();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?;

        let redis_url = env::var("REDIS_URL").ok();

        let host = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("HOST"))?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("PORT"))?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| ConfigError::MissingEnv("JWT_SECRET"))?;

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string()) // Default: 24 hours
            .parse()
            .map_err(|_| ConfigError::InvalidValue("JWT_EXPIRATION"))?;

        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .as_str()
        {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let max_concurrent_tasks = env::var("MAX_CONCURRENT_TASKS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("MAX_CONCURRENT_TASKS"))?;

        Ok(Config {
            database_url,
            redis_url,
            host,
            port,
            jwt_secret,
            jwt_expiration,
            environment,
            log_level,
            max_concurrent_tasks,
        })
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_test(&self) -> bool {
        self.environment == Environment::Test
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),

    #[error("Configuration error: {0}")]
    Custom(String),
}

impl ConfigError {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        ConfigError::Custom(message.as_ref().to_string())
    }
}

pub type ConfigResult<T> = Result<T, ConfigError>;
