use crate::helper::{spawn_app, TestApp};

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

#[tokio::test]
async fn subscribe_returns_200_when_fields_are_present_but_empty() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();
    let tests = vec![
        ("name=&email=monkey%40gmail.com", "empty name"),
        ("name=test%20name&email=", "empty email"),
        ("name=monkey&email=invalid-email-addr", "invalid email"),
    ];

    for (body, msg) in tests {
        let res = client
            .post(&format!("{}/subscriptions", address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            res.status().as_u16(),
            400,
            "The API did not return a 400 BAD REQUEST when payload was {}.",
            msg,
        );
    }
}
