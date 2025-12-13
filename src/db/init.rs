use sqlx::PgPool;

/// Inicializa la tabla `events` en la base de datos `eventack`.
/// Totalmente idempotente: si la tabla existe, no falla.
pub async fn init_database(database_url: &str) -> Result<(), sqlx::Error> {
    // Conectarse a la base de datos eventack como usuario normal
    let pool = PgPool::connect(database_url).await?;

    // Create database if no exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id BIGSERIAL PRIMARY KEY,
            event_id UUID NOT NULL,
            topic TEXT NOT NULL,
            payload JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT now(),
            UNIQUE (topic, event_id)
        )
        "#
    )
    .execute(&pool)
    .await?;

    Ok(())
}
