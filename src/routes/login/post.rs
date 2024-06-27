use actix_web::{http::header, web, HttpResponse};
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::Credentials;

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(skip(form, pool), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(form: web::Form<LoginFormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/"))
        .finish()
}
