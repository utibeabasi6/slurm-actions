use rabbitmq_stream_client::error::StreamCreateError;
use rabbitmq_stream_client::types::{ByteCapacity, ResponseCode};
use rabbitmq_stream_client::{Environment, NoDedup, Producer};

use crate::config::AppConfig;

pub async fn create_rabbitmq_producer(
    config: &AppConfig,
    stream: &str,
    max_length_gb: u64,
) -> Result<Producer<NoDedup>, lib::errors::AppError> {
    let environment = Environment::builder()
        .host(config.rabbitmq_host.clone().as_str())
        .port(config.rabbitmq_port)
        .build()
        .await
        .map_err(|err| lib::errors::AppError::RabbitMQClientError(err))?;

    let create_response = environment
        .stream_creator()
        .max_length(ByteCapacity::GB(max_length_gb))
        .create(stream)
        .await;

    if let Err(e) = create_response {
        if let StreamCreateError::Create { stream, status } = e {
            match status {
                // we can ignore this error because the stream already exists
                ResponseCode::StreamAlreadyExists => {
                    println!("Stream {} already exists, skipping create.", stream);
                }
                err => {
                    println!("Error creating stream: {:?} {:?}", stream, err);
                }
            }
        }
    }

    Ok(environment
        .producer()
        .build(stream)
        .await
        .map_err(|err| lib::errors::AppError::RabbitMQProducerCreateError(err))?)
}
