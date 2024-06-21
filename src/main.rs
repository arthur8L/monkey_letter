use monkey_letter::{
    configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_subscriber(get_subscriber("monkey_letter", "info", std::io::stdout));
    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let conn_pool = PgPool::connect_lazy(config.database.connection_str().expose_secret())
        .expect("Failed to connect to Postgres");
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))?;
    run(listener, conn_pool)?.await
}
