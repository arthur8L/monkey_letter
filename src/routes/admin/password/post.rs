use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use validator::ValidateLength;

use crate::{
    authentication::{self, validate_credentials, Credentials},
    routes::admin::dashboard::get_username,
    session_state::TypedSession,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    confirm_new_password: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session.get_user_id().map_err(e500)?;
    let Some(user_id) = user_id else {
        return Ok(see_other("/login"));
    };
    if form.new_password.expose_secret().length().unwrap_or(0) < 12 {
        FlashMessage::error("New password should be minimum 12 characters.").send();
        return Ok(see_other("/admin/password"));
    }
    if form.new_password.expose_secret() != form.confirm_new_password.expose_secret() {
        FlashMessage::error("You entered mismatching passwords - the field values must match.")
            .send();
        return Ok(see_other("/admin/password"));
    }
    let username = get_username(user_id, &pool).await.map_err(e500)?;

    if let Err(e) = validate_credentials(
        Credentials {
            username,
            password: form.0.current_password,
        },
        &pool,
    )
    .await
    {
        return match e {
            authentication::AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            authentication::AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }
    Ok(HttpResponse::Ok().finish())
}
