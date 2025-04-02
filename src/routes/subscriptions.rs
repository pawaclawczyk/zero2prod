use actix_web::web::Form;
use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use sqlx::PgPool;
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
    log::info!(
        "{} - Adding {} {} as a new subscriber",
        request_id,
        mask(&data.name),
        mask(&data.email),
    );
    log::info!(
        "{} - Saving new subscriber details in the database",
        request_id
    );
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
    .await
    {
        Ok(_) => {
            log::info!("{} - New subscriber details have been saved", request_id);
            HttpResponse::Ok()
        }
        Err(e) => {
            log::error!("{} - Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError()
        }
    }
}
