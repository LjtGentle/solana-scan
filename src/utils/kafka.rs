use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use serde_json;
use anyhow::Result;
use tracing::{info, error};

use crate::config::KafkaConfig;
use crate::models::Transaction;

pub struct KafkaProducer {
    producer: FutureProducer,
    transaction_topic: String,
}

impl KafkaProducer {
    pub async fn new(config: &KafkaConfig) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("client.id", &config.client_id)
            .set("message.timeout.ms", "5000")
            .set("request.required.acks", "1")
            .create()?;

        Ok(Self {
            producer,
            transaction_topic: config.transaction_topic.clone(),
        })
    }

    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<()> {
        let message = serde_json::to_string(transaction)?;
        
        let record = FutureRecord::to(&self.transaction_topic)
            .payload(&message)
            .key(&transaction.signature);

        match self.producer.send(record, rdkafka::util::Timeout::Never).await {
            Ok(delivery) => {
                info!("Transaction sent to Kafka: {:?}", delivery);
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send transaction to Kafka: {}", e);
                Err(KafkaError::MessageProduction(RDKafkaErrorCode::Unknown).into())
            }
        }
    }

    pub async fn send_raw_message(&self, topic: &str, key: &str, payload: &str) -> Result<()> {
        let record = FutureRecord::to(topic)
            .payload(payload)
            .key(key);

        match self.producer.send(record, rdkafka::util::Timeout::Never).await {
            Ok(delivery) => {
                info!("Message sent to Kafka topic {}: {:?}", topic, delivery);
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to send message to Kafka topic {}: {}", topic, e);
                Err(KafkaError::MessageProduction(RDKafkaErrorCode::Unknown).into())
            }
        }
    }
}