use actix_web::rt::spawn;
use uuid::Uuid;

use crate::helper::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn must_be_logged_into_see_the_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn must_be_logged_into_change_password() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let res = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "confirm_new_password": &new_password
        }))
        .await;

    assert_is_redirect_to(&res, "/login")
}

#[tokio::test]
async fn new_password_field_must_match() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // login first
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;
    // change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "confirm_new_password": &another_new_password
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page
        .contains("<p><i>You entered mismatching passwords - the field values must match.</i></p>"))
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let invalid_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &invalid_password,
            "new_password": &new_password,
            "confirm_new_password": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn password_must_be_not_too_short() {
    let app = spawn_app().await;
    let new_password = "too_short".to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "confirm_new_password": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>New password should be minimum 12 characters.</i></p>"));
}
