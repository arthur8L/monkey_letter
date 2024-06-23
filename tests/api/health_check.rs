use crate::helper::{spawn_app, TestApp};

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
