//use uuid::Uuid;
//use serde_json::json;
use dotenv::dotenv;
use std::env;
use eventack::init_database;


#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
    
    init_database(&database_url).await?;

    println!("✅ Database eventack y tabla events listas");
    

    Ok(())
}




/* 
    let event = Event {
        event_id: Uuid::now_v7(),  // ← UUID v7
        topic: "user.registered".to_string(),
        payload: json!({
            "user_id": 42,
            "email": "user@example.com"
        }),
    };

    let inserted = save_event_tx(&pool, &event).await?;

    if inserted {
        println!("Evento insertado (commit OK)");
    } else {
        println!("Evento duplicado — idempotencia funcionando");
    }
*/
