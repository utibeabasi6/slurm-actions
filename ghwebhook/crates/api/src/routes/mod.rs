use crate::types::AppState;
use lib::types::githubevent::GithubEvent;
use rabbitmq_stream_client::types::Message;
use rocket::{State, http::Status, serde::json::Json};

#[post("/webhook", data = "<payload>", format = "application/json")]
pub async fn webhook(payload: Json<GithubEvent>, state: &State<AppState>) -> Status {
    let payload_bytes = match serde_json::to_vec(&payload.into_inner()) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Failed to serialize payload: {}", err);
            return Status::InternalServerError;
        }
    };
    let confirmation_status = match state
        .rabbitmq_producer
        .send_with_confirm(Message::builder().body(payload_bytes).build())
        .await
    {
        Ok(status) => status,
        Err(err) => {
            eprintln!("Failed to send message to RabbitMQ: {}", err);
            return Status::InternalServerError;
        }
    };

    if !confirmation_status.confirmed() {
        eprintln!(
            "Message not confirmed. Status: {:?}",
            confirmation_status.status()
        );
        return Status::InternalServerError;
    }

    Status::Ok
}
