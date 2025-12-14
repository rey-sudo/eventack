use crate::models::event::CreateEventRequest;
use crate::models::event::Event;
use crate::models::event::SerializedEvent;
use crate::utils::event::serialize_event;
use actix_web::{HttpResponse, Responder, post, web};
use sqlx::PgPool;
use sqlx::Row;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Http POST handler that creates a new event. The event content (payload) is serialized with binary CBOR, compressed with LZ4, and hashed with SHA256.
#[post("/events")]
pub async fn create_event(
    pool: web::Data<PgPool>,
    req: web::Json<CreateEventRequest>,
) -> impl Responder {
    let now_ms: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let event_id: Uuid = Uuid::now_v7();

    let event: Event = Event {
        user_id: req.user_id.clone(),
        action: req.action.clone(),
        value: req.value, //Copy
    };

    let serialized: SerializedEvent = match serialize_event(&event) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error serializando el payload: {:?}", err);
            return HttpResponse::InternalServerError().body("Error serializando payload");
        }
    };

    let result: Result<Option<sqlx::postgres::PgRow>, sqlx::Error> = sqlx::query(
        r#"
        INSERT INTO events (event_id, topic, payload, payload_hash, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (topic, event_id) DO NOTHING
        RETURNING event_id
        "#,
    )
    .bind(event_id)
    .bind(&req.topic)
    .bind(&serialized.payload) // CBOR + LZ4
    .bind(&serialized.hash) // SHA-256 hash
    .bind(now_ms)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let id: Uuid = row.get("event_id");
            HttpResponse::Created().json(id)
        }
        Ok(None) => HttpResponse::Ok().json(event_id), //idempotent
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use serde_json::json;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn post_events_creates_event(pool: PgPool) {
        // Ejecutar migraciones
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Inicializar app con handler
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(pool.clone()))
                .service(create_event),
        )
        .await;

        // Crear payload para enviar al handler
        let payload = json!({
            "topic": "order.created",
            "user_id": "user123",
            "action": "click",
            "value": 42
        });

        // TestRequest HTTP POST
        let req = test::TestRequest::post()
            .uri("/events")
            .set_json(&payload)
            .to_request();

        // Llamar al servicio
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Verificar que el evento se insertó en la DB
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE topic = $1")
            .bind("order.created")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.0, 1);
    }

    #[sqlx::test]
    async fn post_events_is_idempotent(pool: PgPool) {
        // Ejecutar migraciones
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Inicializar app con handler
        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(pool.clone()))
                .service(create_event),
        )
        .await;

        // Payload JSON que se enviará al handler
        let payload = json!({
            "topic": "order.created",
            "user_id": "user123",
            "action": "click",
            "value": 42
        });

        // Llamar al handler varias veces
        for _ in 0..5 {
            let req = test::TestRequest::post()
                .uri("/events")
                .set_json(&payload)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success()); // HTTP 201 o 200
        }

        // Verificar cuántos eventos existen con el mismo topic
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE topic = $1")
            .bind("order.created")
            .fetch_one(&pool)
            .await
            .unwrap();

        // Cada llamada genera un event_id único → deben existir 5 filas
        assert_eq!(count.0, 5);
    }
}
