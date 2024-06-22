use monkey_letter::{
    configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_subscriber(get_subscriber("monkey_letter", "info", std::io::stdout));
    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let conn_pool = PgPool::connect_lazy_with(config.database.with_db());
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout,
    );
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))?;
    run(listener, conn_pool, email_client)?.await
}
