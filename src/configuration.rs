//! src/configuration.rs
use crate::routes::subscriptions::SubscriberEmail;
use serde::{Deserialize, Serialize};
use std::env;

/// Application settings loaded from environment variables
#[derive(Debug, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
    pub email_settings: EmailSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub sender_email: SubscriberEmail,
    pub service_url: String,
    pub api_token: String,
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

        let email_settings = EmailSettings {
            sender_email: SubscriberEmail {
                email: env::var("APP__EMAIL__SENDER").expect("APP__EMAIL__SENDER not set"),
            },
            service_url: env::var("APP__EMAIL__SERVICE_URL")
                .expect("APP__EMAIL__SERVICE_URL not set"),
            api_token: env::var("APP__EMAIL__API_TOKEN").expect("APP__EMAIL__API_TOKEN not set"),
        };

        Settings {
            database,
            application_port,
            email_settings,
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
