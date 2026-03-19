use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct EventActor {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct EventItem {
    pub event_id: Uuid,
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub actor: Option<EventActor>,
    #[schema(value_type = Object)]
    pub payload: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct ListEventsResponse {
    pub items: Vec<EventItem>,
}
