use std::net::TcpListener;

use monkey_letter::configuration::get_configuration;
use sqlx::{Connection, PgConnection};

#[tokio::test]
async fn health_check_test() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Failed to excute request to health_check");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app();
    let config = get_configuration().expect("Failed to read configuration.");
    let conn_str = config.database.connection_str();
    let mut db_conn = PgConnection::connect(&conn_str)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    let body = "name=monkey%20struct&email=monkeystruct_test%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute post request");

    //assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);
    assert_eq!(200, response.status().as_u16());

    //check db for entry
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut db_conn)
        .await
        .expect("Failed to fetch saved subscrib ption");
    assert_eq!(saved.email, "monkeystruct_test@gmail.com");
    assert_eq!(saved.name, "monkey struct");
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=monkey%man", "Missing the email"),
        ("email=monk%40gmail.com", "Missing the name"),
        ("", "Missing Both"),
    ];

    for (query, msg) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &addr))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let monkey_serv = monkey_letter::startup::run(listener).expect("Failed to bind address");

    tokio::spawn(monkey_serv);
    format!("http://127.0.0.1:{}", port)
}
