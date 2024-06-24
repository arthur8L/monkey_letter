use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(query, db_pool))]
pub async fn confirm(db_pool: web::Data<PgPool>, query: web::Query<Parameters>) -> HttpResponse {
    let Ok(sub_id) = get_subscription_id_from_token(&db_pool, &query.subscription_token).await
    else {
        return HttpResponse::InternalServerError().finish();
    };
    let Some(id) = sub_id else {
        return HttpResponse::Unauthorized().finish();
    };
    match confirm_subscriber(&db_pool, id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed executing a query: {:?}", e);
        e
    })?;
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(res.map(|r| r.subscriber_id))
}
