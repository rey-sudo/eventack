use sqlx::PgPool;

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
    payload BYTEA NOT NULL,      
    payload_hash BYTEA NOT NULL,  
    created_at BIGINT NOT NULL,   
    UNIQUE (topic, event_id)
)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_topic ON events(topic)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at)")
        .execute(&pool)
        .await?;

    println!("✅ Database eventack y tabla events listas");

    Ok(())
}
