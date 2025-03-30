use std::net::TcpListener;

#[tokio::test]
async fn test_health_check() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/subscribe", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=john%40example.com&name=John%20Doe")
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("", "missing both email and name"),
        ("name=John%20Doe", "missing email"),
        ("email=john%40example.com", "missing name"),
    ];

    for (data, expected_error) in test_cases {
        let response = client
            .post(format!("{}/subscribe", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(data)
            .send()
            .await
            .expect("Failed to send request.");

        assert_eq!(
            response.status(),
            400,
            "The API didn't fail with 400 Bad Request when payload was {}",
            expected_error
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    let handle = tokio::spawn(server);
    drop(handle);
    format!("http://127.0.0.1:{}", port)
}
