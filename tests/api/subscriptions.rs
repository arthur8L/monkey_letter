use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helper::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=monkey%20struct&email=monkeystruct_test%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    //assert_eq!(response.status(), reqwest::StatusCode::ACCEPTED);
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;

    let body = "name=monkey%20struct&email=monkeystruct_test%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscrib ption");
    assert_eq!(saved.email, "monkeystruct_test@gmail.com");
    assert_eq!(saved.name, "monkey struct");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_form_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=monkey%man", "Missing the email"),
        ("email=monk%40gmail.com", "Missing the name"),
        ("", "Missing Both"),
    ];

    for (query, msg) in test_cases {
        let resp = app.post_subscriptions(query.into()).await;
        assert_eq!(
            400,
            resp.status().as_u16(),
            "Expected API to fail returning 400 Bad Request for {}",
            msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let tests = vec![
        ("name=&email=monkey%40gmail.com", "empty name"),
        ("name=test%20name&email=", "empty email"),
        ("name=monkey&email=invalid-email-addr", "invalid email"),
    ];

    for (body, msg) in tests {
        let res = app.post_subscriptions(body.into()).await;
        assert_eq!(
            res.status().as_u16(),
            400,
            "The API did not return a 400 BAD REQUEST when payload was {}.",
            msg,
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_form_valid_submit() {
    let app = spawn_app().await;
    let body = "name=monkey%20struct&email=monkeystruct%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=monkey%20struct&email=monkeystruct%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_req = &app.email_server.received_requests().await.unwrap()[0];

    let link = app.get_confirmation_link(email_req);

    assert_eq!(link.html, link.plain_text);
}
