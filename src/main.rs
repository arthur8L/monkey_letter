use std::net::TcpListener;

use monkey_letter::run;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed binding to port 8000");
    run(listener)?.await
}
