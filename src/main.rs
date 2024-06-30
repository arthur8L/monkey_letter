use std::fmt::{Debug, Display};

use monkey_letter::{
    configuration,
    issue_delivery_worker::run_worker_until_stopped,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry initializaer
    init_subscriber(get_subscriber("monkey_letter", "info", std::io::stdout));
    let config = configuration::get_configuration().expect("Failed to read configuration.");

    let server = Application::build(config.clone()).await?;
    let server_task = tokio::spawn(server.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(config));
    tokio::select! {
        o = server_task => report_exit("API", o),
        o = worker_task => report_exit("Background_worker", o)
    }
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete",
                task_name
            )
        }
    }
}
