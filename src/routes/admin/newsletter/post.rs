use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(name = "Publish a newsletter issue", skip(form, user_id, pool, email_client) fields(user_id=%*user_id))]
pub async fn send_newsletter(
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscriber(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.title,
                        &form.html_content,
                        &form.text_content,
                    )
                    .await
                    .map_err(e500)?;
            }
            Err(err) => {
                tracing::warn!(err.cause_chain = ?err, error.message = ?err, "Skipping a confirmed subscriber with invalid stored data")
            }
        }
    }
    FlashMessage::info("The newletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
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
