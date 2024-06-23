use monkey_letter::{
    configuration::{self, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let (name, env_filter) = ("test", "debug");
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(name, env_filter, std::io::stdout));
    } else {
        init_subscriber(get_subscriber(name, env_filter, std::io::sink));
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = {
        let mut c = configuration::get_configuration().expect("Failed to load config");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };
    configure_database(&config.database).await;
    let application = Application::build(config.clone())
        .await
        .expect("Failed to build application");
    let port = application.port();
    tokio::spawn(application.run_until_stopped());
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: get_connection_pool(&config.database),
    }
}
//clean up is not implemented. probably better to do so.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
