use std::net::TcpListener;

use monkey_letter::{configuration, startup::run};
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("monkey_letter".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set Tracing Subscriber");

    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let conn_pool = PgPool::connect(&config.database.connection_str())
        .await
        .expect("Failed to connect to Postgres");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    run(listener, conn_pool)?.await
}
