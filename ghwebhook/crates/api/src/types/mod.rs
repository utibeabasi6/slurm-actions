use rabbitmq_stream_client::{NoDedup, Producer};

pub struct AppState {
    pub rabbitmq_producer: Producer<NoDedup>,
}
