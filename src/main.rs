use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration, server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = configuration::get_configuration().expect("Failed to read config");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let address = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;
    server::run(address, connection_pool)?.await
}
