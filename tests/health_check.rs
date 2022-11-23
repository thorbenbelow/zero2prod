use std::fmt::format;
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let ip = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", ip)).expect("Failed to bind");
    let port = listener
        .local_addr()
        .expect("Failed to retrieve port")
        .port();
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);
    format!("http://{}:{}", ip, port)
}
