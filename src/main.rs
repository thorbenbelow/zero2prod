use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration, server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = configuration::get_configuration().expect("Failed to read config");

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let address = TcpListener::bind(format!(
        "{}:{}",
        configuration.app.host, configuration.app.port
    ))?;
    server::run(address, connection_pool)?.await
}
