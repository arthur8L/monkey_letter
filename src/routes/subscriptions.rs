use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let req_id = Uuid::new_v4();
    let req_span = tracing::info_span!("Adding a new subscriber", %req_id, subscriber_name = %form.name, subscriber_email=%form.email);
    let _req_span_guard = req_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} -New Subscriber is succeessfully added",
                req_id
            );
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            tracing::error!("request_id {} - Failed to execute query: {:?}", req_id, err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
