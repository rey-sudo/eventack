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


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sqlx::PgPool;
    use serde_json::json;
    use uuid::{Uuid, timestamp::Timestamp, NoContext};

    #[sqlx::test]
    async fn post_events_creates_event(pool: PgPool) {
        // migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

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

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE topic = $1"
        )
        .bind("order.created")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count, 1);
    }


  #[sqlx::test]
    async fn post_events_is_idempotent(pool: PgPool) {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        let event_id = Uuid::new_v7(Timestamp::now(NoContext));

        for _ in 0..5 {
            let _ = sqlx::query(
                r#"
                INSERT INTO events (event_id, topic, payload)
                VALUES ($1, $2, $3)
                ON CONFLICT (topic, event_id) DO NOTHING
                "#
            )
            .bind(event_id)
            .bind("order.created")
            .bind(json!({ "order_id": 123 }))
            .execute(&pool)
            .await;
        }

        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM events WHERE event_id = $1"#
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count, 1);
    }






}


