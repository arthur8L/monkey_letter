use std::net::TcpListener;

use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, dev::Server, web, App, HttpServer};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};
use actix_web_lab::middleware::from_fn;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

use crate::{
    authentication::reject_annonymousr_user,
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{
        admin_dashboard, change_password, change_password_form, confirm, health_check, home, login,
        login_form, logout, send_newsletter, send_newsletter_form, subscribe,
    },
};

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        let db_pool = get_connection_pool(&config.database);
        let email_client = config.email_client.client();
        let listener = TcpListener::bind(format!(
            "{}:{}",
            config.application.host, config.application.port
        ))?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool,
            email_client,
            config.application.base_url,
            config.application.hmac_secret,
            config.redis_url,
        )
        .await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_url: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let email_client = web::Data::new(email_client);
    let connection = web::Data::new(db_pool);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    // Flash Message Middleware
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_storage = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_storage).build();
    let redis_store = RedisSessionStore::new(redis_url.expose_secret()).await?;
    let server = HttpServer::new(move || {
        // Route::new().guard(guard::Get()) == web::get()
        App::new()
            // wrap is used for middleware
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/health_check", web::get().to(health_check))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_annonymousr_user))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/newsletters", web::get().to(send_newsletter_form))
                    .route("/newsletters", web::post().to(send_newsletter))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(logout)),
            )
            .app_data(connection.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}
