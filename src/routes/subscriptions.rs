use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::new_subscriber::NewSubscriber;

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    name: String,
    email: String,
}

impl TryFrom<SubscriptionFormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscriptionFormData) -> Result<Self, Self::Error> {
        NewSubscriber::parse(value.name, value.email)
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
name = "Adding a new subscriber",
skip(form, conn),
fields(
subscriber_email = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<SubscriptionFormData>,
    conn: web::Data<PgPool>,
) -> HttpResponse {
    let new_sub = match form.0.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&new_sub, &conn).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, conn)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    conn: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(conn)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
