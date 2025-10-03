use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error launching rocket: {0}")]
    RocketError(rocket::Error),
    #[error("Error parsing configuration: {0}")]
    ConfigError(config::ConfigError),
    #[error("RabbitMQ client error: {0}")]
    RabbitMQClientError(rabbitmq_stream_client::error::ClientError),
    #[error("Error creating producer: {0}")]
    RabbitMQProducerCreateError(rabbitmq_stream_client::error::ProducerCreateError),
    #[error("Error cloning git repository: {0}")]
    GitCloneError(String),
    #[error("Error creating temporary directory: {0}")]
    TempDirCreationError(String),
    #[error("Error creating consumer: {0}")]
    RabbitMQConsumerCreateError(rabbitmq_stream_client::error::ConsumerCreateError),
    #[error("Error closing consumer: {0}")]
    RabbitMQConsumerCloseError(rabbitmq_stream_client::error::ConsumerCloseError),
    #[error("Error consuming message: {0}")]
    RabbitMQConsumerConsumeError(String),
}

impl From<rocket::Error> for AppError {
    fn from(err: rocket::Error) -> Self {
        AppError::RocketError(err)
    }
}
