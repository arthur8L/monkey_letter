use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_query))]
pub async fn confirm(_query: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
