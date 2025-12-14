CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL,
    topic TEXT NOT NULL,
    payload BYTEA NOT NULL,      
    payload_hash BYTEA NOT NULL,  
    created_at BIGINT NOT NULL,   
    UNIQUE (topic, event_id)
)