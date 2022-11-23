use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::{PgConnection, PgPool};

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, conn_pool: PgPool) -> Result<Server, std::io::Error> {
    let conn_pool = web::Data::new(conn_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(conn_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
