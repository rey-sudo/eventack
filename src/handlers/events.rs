use actix_web::{HttpResponse, Responder, post, web};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::event::CreateEventRequest;
use crate::models::event::Event;
use crate::utils::event::{serialize_event};
use crate::models::event::SerializedEvent;


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
    use uuid::{NoContext, Uuid, timestamp::Timestamp};

    #[sqlx::test]
    async fn post_events_creates_event(pool: PgPool) {
        // migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let app = test::init_service(
            App::new()
                .app_data(actix_web::web::Data::new(pool.clone()))
                .service(create_event),
        )
        .await;

        let event_id = Uuid::new_v7(Timestamp::now(NoContext));

        let payload = json!({
            "event_id": event_id,
            "topic": "order.created",
            "payload": {
                "order_id": 123,
                "amount": 49.99
            }
        });

        let req = test::TestRequest::post()
            .uri("/events")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE topic = $1")
            .bind("order.created")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn post_events_is_idempotent(pool: PgPool) {
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let event_id = Uuid::new_v7(Timestamp::now(NoContext));

        for _ in 0..5 {
            let _ = sqlx::query(
                r#"
                INSERT INTO events (event_id, topic, payload)
                VALUES ($1, $2, $3)
                ON CONFLICT (topic, event_id) DO NOTHING
                "#,
            )
            .bind(event_id)
            .bind("order.created")
            .bind(json!({ "order_id": 123 }))
            .execute(&pool)
            .await;
        }

        let count: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM events WHERE event_id = $1"#)
            .bind(event_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }
}
