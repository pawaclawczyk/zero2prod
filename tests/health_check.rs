use sqlx::{Connection, Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, Settings, get_configuration};

pub struct TestApp {
    pub address: String,
    pub connection_pool: PgPool,
    pub configuration: Settings,
}

#[tokio::test]
async fn test_health_check() {
    let app = set_up_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = set_up_app().await;
    let client = reqwest::Client::new();

    let body = "email=john%40example.com&name=John%20Doe";

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request.");

    assert_eq!(200, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!("john@example.com", saved.email);
    assert_eq!("John Doe", saved.name);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = set_up_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("", "missing both email and name"),
        ("name=John%20Doe", "missing email"),
        ("email=john%40example.com", "missing name"),
    ];

    for (data, expected_error) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
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

async fn set_up_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    let database_name = Uuid::new_v4().to_string();
    configuration.database.database_name = database_name;

    let connection_pool = set_up_database(&configuration.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address.");

    let handle = tokio::spawn(server);
    drop(handle);

    TestApp {
        address,
        connection_pool,
        configuration,
    }
}

async fn set_up_database(configuration: &DatabaseSettings) -> PgPool {
    let mut connection =
        sqlx::PgConnection::connect(&configuration.connection_string_without_database())
            .await
            .expect("Failed to connect to database.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, configuration.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = sqlx::PgPool::connect(&configuration.connection_string())
        .await
        .expect("Failed to connect to database.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

    pool
}
