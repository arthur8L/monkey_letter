use crate::helper::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_msg_is_set_on_failure() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": "invalid_username",
        "password": "invalid_password"
    });

    let response = app.post_login(&body).await;

    assert_is_redirect_to(&response, "/login")
}
