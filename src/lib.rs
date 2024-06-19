pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::{dev::Server, web, App, HttpServer};
use routes::{health_check, subscribe};
use std::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        // Route::new().guard(guard::Get()) == web::get()
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
