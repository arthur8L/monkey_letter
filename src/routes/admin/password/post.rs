use actix_web::HttpResponse;

pub async fn change_password() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}
