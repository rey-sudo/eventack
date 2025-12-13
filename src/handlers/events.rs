use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use sqlx::Row;

use crate::models::event::CreateEventRequest;

#[post("/events")]
pub async fn create_event(
    pool: web::Data<PgPool>,
    req: web::Json<CreateEventRequest>,
) -> impl Responder {
    let event_id = Uuid::now_v7();

    let result = sqlx::query(
        r#"
        INSERT INTO events (event_id, topic, payload)
        VALUES ($1, $2, $3)
        ON CONFLICT (topic, event_id) DO NOTHING
        RETURNING event_id
        "#
    )
    .bind(event_id)
    .bind(&req.topic)
    .bind(&req.payload)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let id: Uuid = row.get("event_id");
            HttpResponse::Created().json(id)
        }
        Ok(None) => {
            // Evento duplicado (idempotencia)
            HttpResponse::Ok().json(event_id)
        }
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
