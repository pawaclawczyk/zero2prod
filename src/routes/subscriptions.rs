use actix_web::web::Form;
use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

fn mask(text: &str) -> String {
    let prefix: String = text.graphemes(true).take(3).collect();
    format!("{}{}", prefix, "*".repeat(7))
}

pub async fn subscribe(data: Form<SubscribeForm>, connection: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    let span = tracing::info_span!("subscribe", %request_id, email = %mask(&data.email), name = %mask(&data.name));
    let _span_guard = span.enter();

    tracing::info!(
        "{} - Adding {} {} as a new subscriber",
        request_id,
        mask(&data.name),
        mask(&data.email),
    );
    tracing::info!(
        "{} - Saving new subscriber details in the database",
        request_id
    );

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data.email,
        data.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("{} - New subscriber details have been saved", request_id);
            HttpResponse::Ok()
        }
        Err(e) => {
            tracing::error!("{} - Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError()
        }
    }
}
