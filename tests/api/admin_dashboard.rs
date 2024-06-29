use crate::helper::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn must_be_logged_into_access_admin_panel() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    // login;
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // follow redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains("You have successfully logged out."));

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}
