use std::net::TcpListener;

use monkey_letter::{configuration, startup::run};
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = configuration::get_configuration().expect("Failed to read configuration.");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))?;
    run(listener)?.await
}
