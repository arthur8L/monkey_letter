use actix_web::{web, HttpResponse};
use sqlx::PgPool;

pub async fn publish_newsletter(_pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
