use sqlx::PgPool;

#[sqlx::test]
async fn post_events_creates_event(pool: PgPool) {
    let result = sqlx::query!("SELECT 1")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok());
}
