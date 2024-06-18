use std::net::TcpListener;

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let monkey_serv = monkey_letter::run(listener).expect("Failed to bind address");

    tokio::spawn(monkey_serv);
    format!("http://127.0.0.1:{}", port)
}
