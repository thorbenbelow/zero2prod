use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

use crate::domain::subscriber_email::SubscriberEmail;

pub enum Environment {
    Dev,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Environment::Dev),
            "prod" => Ok(Environment::Prod),
            other => Err(format!(
                "{} is not a supported environment! Use 'dev' or 'prod' instead.",
                other
            )),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(log::LevelFilter::Trace);
        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(match self.require_ssl {
                true => PgSslMode::Require,
                false => PgSslMode::Disable,
            })
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "dev".to_string())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let settings = config::Config::builder()
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;
    settings.try_deserialize::<Settings>()
}
