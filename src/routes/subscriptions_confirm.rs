use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use super::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
#[tracing::instrument(name = "Confirm a pending subscriber", skip(query, db_pool))]
pub async fn confirm(
    db_pool: web::Data<PgPool>,
    query: web::Query<Parameters>,
) -> Result<HttpResponse, ConfirmError> {
    let sub_id = get_subscription_id_from_token(&db_pool, &query.subscription_token)
        .await
        .context("Failed to get subscription id from token")?;
    let Some(id) = sub_id else {
        return Ok(HttpResponse::Unauthorized().finish());
    };
    confirm_subscriber(&db_pool, id)
        .await
        .context("Failed to confirm subscriber")?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_subscription_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let res = sqlx::query!(
        r#"SELECT subscriber_id from subscription_tokens WHERE subscription_token = $1"#,
        token
    )
    .fetch_optional(pool)
    .await?;
    Ok(res.map(|r| r.subscriber_id))
}
