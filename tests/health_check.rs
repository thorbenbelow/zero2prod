use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::spawn;
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let ip = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", ip)).expect("Failed to bind");
    let mut configuration = get_configuration().expect("Failed to read config");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;
    let port = listener
        .local_addr()
        .expect("Failed to retrieve port")
        .port();
    let server =
        zero2prod::server::run(listener, db_pool.clone()).expect("Failed to bind address.");
    let _ = spawn(server);
    let address = format!("http://{}:{}", ip, port);
    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to postgres");

    connection
        .execute(format!(r#"Create DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create db");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate");

    db_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=bobby%20B&email=bobbyb%40test.invalid";

    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch subscriptions");
    assert_eq!(saved.email, "bobbyb@test.invalid");
    assert_eq!(saved.name, "bobby B");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=bobby%20B", "missing email"),
        ("email=bobbyb%40test.invalid", "missing name"),
        ("", "missing email and name"),
    ];
    for (body, error) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 when the payload was {}",
            error
        );
    }
}
