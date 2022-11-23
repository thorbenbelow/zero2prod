use std::fmt::format;
use std::net::TcpListener;
use tokio::spawn;

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=bobby%20B&email=bobbyb%40test.invalid";
    let response = client
        .post(format!("{}/subscriptions", app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=bobby%20B", "missing email"),
        ("email=bobbyb%40test.invalid", "missing name"),
        ("", "missing email and name"),
    ];
    for (body, error) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app_address))
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
