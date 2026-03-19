use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct EventActor {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct EventItem {
    pub event_id: Uuid,
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub actor: Option<EventActor>,
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct ListEventsResponse {
    pub items: Vec<EventItem>,
}
