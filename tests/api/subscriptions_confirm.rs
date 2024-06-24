use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helper::spawn_app;

#[tokio::test]
async fn confirmation_without_token_rejected_with_400() {
    let app = spawn_app().await;

    let res = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed sending request");

    assert_eq!(res.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_sub_returns_200_when_called() {
    let app = spawn_app().await;
    let body = "name=monk%20struct&email=monkey%40test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let link = app.get_confirmation_link(email_request);

    let res = reqwest::get(link.html).await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=monk%20struct&email=monkey%40test.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_req = &app.email_server.received_requests().await.unwrap()[0];
    let link = app.get_confirmation_link(email_req);

    reqwest::get(link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch confirmed subscription.");

    assert_eq!(saved.email, "monkey@test.com");
    assert_eq!(saved.name, "monk struct");
    assert_eq!(saved.status, "confirmed");
}
