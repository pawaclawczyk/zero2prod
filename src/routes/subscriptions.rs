use actix_web::{HttpResponse, web};
use chrono;
use sqlx;
use tracing;
use unicode_segmentation::UnicodeSegmentation;
use uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

fn mask(text: &str) -> String {
    let prefix: String = text.graphemes(true).take(3).collect();
    format!("{}{}", prefix, "*".repeat(7))
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(data, connection),
    fields(
        subscriber_email = %mask(&data.email),
        subscriber_name = %mask(&data.name),
    )
)]
pub async fn subscribe(
    data: web::Form<SubscribeForm>,
    connection: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    match insert_subscriber(&data, &connection).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(data, connection)
)]
async fn insert_subscriber(
    data: &SubscribeForm,
    connection: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        data.email,
        data.name,
        chrono::Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
