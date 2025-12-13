use sqlx::{PgPool, Row};

/// Inicializa la base de datos `eventack` y la tabla `events`.
/// Es totalmente idempotente:
/// - Si la DB existe, no falla
/// - Si la tabla existe, no falla
pub async fn init_database(admin_db_url: &str) -> Result<(), sqlx::Error> {
    // 1. Conectarse a la DB administrativa (normalmente "postgres")
    let admin_pool = PgPool::connect(admin_db_url).await?;

    // 2. Crear la base de datos si no existe
    let db_exists: bool = sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM pg_database WHERE datname = 'eventack'
        )
        "#
    )
    .fetch_one(&admin_pool)
    .await?
    .get(0);

    if !db_exists {
        // CREATE DATABASE no puede ejecutarse dentro de una transacción
        sqlx::query("CREATE DATABASE eventack")
            .execute(&admin_pool)
            .await?;
    }

    // 3. Conectarse a la base de datos eventack
    let eventack_url = admin_db_url.replace("/postgres", "/eventack");
    let eventack_pool = PgPool::connect(&eventack_url).await?;

    // 4. Crear la tabla events si no existe
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
    .execute(&eventack_pool)
    .await?;

    Ok(())
}
