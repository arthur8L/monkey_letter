use actix_web::{
    http::{
        header::{self, HeaderMap, HeaderValue},
        StatusCode,
    },
    web, HttpRequest, HttpResponse, ResponseError,
};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::SubscriberEmail, email_client::EmailClient, telemetry::spawn_blocking_with_tracing,
};

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<Uuid, PublishError> {
    let (user_id, expected_password_hash) = get_stored_credentials(&credentials.username, pool)
        .await
        .map_err(PublishError::UnexpectedError)?
        .ok_or_else(|| PublishError::AuthError(anyhow::anyhow!("Unknown username.")))?;

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)??;
    Ok(user_id)
}

#[tracing::instrument(name = "Verify password hash", skip(expected_password_hash, password))]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse PHC string format")
        .map_err(PublishError::UnexpectedError)?;
    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &expected_password_hash)
        .context("Invalid password")
        .map_err(PublishError::AuthError)
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
pub async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"SELECT user_id, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve credential")?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));
    Ok(row)
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(pool, body, email_client,request)
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    pool: web::Data<PgPool>,
    body: web::Json<BodyData>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(user_id));
    let subscribers = get_confirmed_subscriber(&pool)
        .await
        .context("Failed getting confirmed subscriber")?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter to {}", subscriber.email)
                    })?;
            }
            Err(err) => {
                tracing::warn!(err.cause_chain = ?err, "Skipping a confirmed subscriber with invalid stored data")
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}
#[tracing::instrument(name = "Get Confirmed Subscriber", skip(pool))]
async fn get_confirmed_subscriber(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!(r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|v| match SubscriberEmail::parse(v.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(error) => Err(anyhow::anyhow!(error)),
            })
            .collect();
    Ok(confirmed_subscribers)
}
struct Credentials {
    username: String,
    password: Secret<String>,
}
fn basic_authentication(header: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = header
        .get("Authorization")
        .context("The \"Authorization\" header is missing")?
        .to_str()
        .context("The Authorization header was not valid UTF8 string")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization header is now basic format")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to decode base64 'Basic' credentials")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8")?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'basic' format"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password should be provided in 'basic' format"))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
