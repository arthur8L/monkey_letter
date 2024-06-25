use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helper::{spawn_app, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    // To Assert No req fired to email service
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_req_body = serde_json::json!({
        "title": "Newsletter Title",
        "content": {
            "text": "Newsletter in plain text",
            "html": "<h1>Newsletter</h1> as HTML"
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletter_req_body)
        .send()
        .await
        .expect("Failed to send req to newsletter");

    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = "name=monkey&email=monk%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}
