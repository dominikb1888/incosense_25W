//! src/configuration.rs
use std::env;

/// Application settings loaded from environment variables
#[derive(Debug, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl Settings {
    /// Load settings from environment variables
    /// Panics if any required variable is missing or invalid
    pub fn from_env() -> Self {
        let database = DatabaseSettings {
            username: env::var("APP__DATABASE__USERNAME").expect("APP__DATABASE__USERNAME not set"),
            password: env::var("APP__DATABASE__PASSWORD").expect("APP__DATABASE__PASSWORD not set"),
            host: env::var("APP__DATABASE__HOST").expect("APP__DATABASE__HOST not set"),
            port: env::var("APP__DATABASE__PORT")
                .expect("Missing APP__DATABASE__PORT")
                .parse()
                .expect("APP__DATABASE__PORT must be a number"),
            database_name: env::var("APP__DATABASE__DATABASE_NAME")
                .expect("APP__DATABASE__DATABASE_NAME not set"),
        };

        let application_port = env::var("APP__APPLICATION_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .expect("APP__APPLICATION_PORT must be a valid u16");

        Settings {
            database,
            application_port,
        }
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
