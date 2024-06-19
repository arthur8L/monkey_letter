use std::net::TcpListener;

use env_logger::Env;
use monkey_letter::{configuration, startup::run};
use sqlx::PgPool;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let conn_pool = PgPool::connect(&config.database.connection_str())
        .await
        .expect("Failed to connect to Postgres");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    run(listener, conn_pool)?.await
}
