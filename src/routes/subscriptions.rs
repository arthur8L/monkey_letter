use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let req_id = Uuid::new_v4();
    log::info!(
        "request_id {} - Adding username: {}, email: {} as a new subscriber.",
        req_id,
        form.name,
        form.email
    );
    log::info!("Saving new subscriber details in the database");
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
    .await
    {
        Ok(_) => {
            log::info!(
                "request_id {} -New Subscriber is succeessfully added",
                req_id
            );
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            log::error!("request_id {} - Failed to execute query: {:?}", req_id, err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
