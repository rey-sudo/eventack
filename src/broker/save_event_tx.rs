use sqlx::{PgPool, Postgres, Transaction, postgres::PgQueryResult};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Event {
    pub event_id: Uuid,   // ahora UUID v7
    pub topic: String,
    pub payload: Value,
}

/// Inserta un evento en PostgreSQL de forma idempotente,
/// usando UUID v7 como identificador, dentro de una transacción.
///
/// - Si el evento NO existía → lo inserta y retorna true.
/// - Si el evento YA existía → no hace nada y retorna false.
pub async fn save_event_tx(pool: &PgPool, event: &Event) -> Result<bool, sqlx::Error> {
    // 1. Iniciar transacción
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 2. Ejecutar el INSERT dentro de la transacción
    let result: PgQueryResult = sqlx::query(
        r#"
        INSERT INTO events (event_id, topic, payload)
        VALUES ($1, $2, $3)
        ON CONFLICT (topic, event_id) DO NOTHING
        "#,
    )
    .bind(event.event_id)
    .bind(&event.topic)
    .bind(&event.payload)
    .execute(&mut tx)
    .await?;

    let inserted = result.rows_affected() == 1;

    // 3. Hacer commit (todo es atómico)
    tx.commit().await?;

    Ok(inserted)
}
