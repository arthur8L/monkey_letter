use monkey_letter::{
    configuration::{self, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let (name, env_filter) = ("test", "debug");
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(name, env_filter, std::io::stdout));
    } else {
        init_subscriber(get_subscriber(name, env_filter, std::io::sink));
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_test() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to excute request to health_check");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let TestApp { address, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=monkey%20struct&email=monkeystruct_test%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute post request");

    //assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);
    assert_eq!(200, response.status().as_u16());

    //check db for entry
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscrib ption");
    assert_eq!(saved.email, "monkeystruct_test@gmail.com");
    assert_eq!(saved.name, "monkey struct");
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_form_data() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=monkey%man", "Missing the email"),
        ("email=monk%40gmail.com", "Missing the name"),
        ("", "Missing Both"),
    ];

    for (query, msg) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(query)
            .send()
            .await
            .expect("Failed to execute post request");
        assert_eq!(
            400,
            resp.status().as_u16(),
            "Expected API to fail returning 400 Bad Request for {}",
            msg
        );
    }
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    let mut config = configuration::get_configuration().expect("Failed to load config");
    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.database).await;
    let monkey_serv =
        monkey_letter::startup::run(listener, db_pool.clone()).expect("Failed to bind address");

    tokio::spawn(monkey_serv);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}
//clean up is not implemented. probably better to do so.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
