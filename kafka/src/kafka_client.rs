use std::time::Duration;

use rdkafka::{
    config::FromClientConfig,
    consumer::{Consumer, DefaultConsumerContext, MessageStream, StreamConsumer},
    producer::{future_producer::OwnedDeliveryResult, FutureProducer, FutureRecord},
    util::Timeout,
    ClientConfig,
};

pub struct KafkaClient {
    client_config: ClientConfig,
    timeout: Timeout,
}

// bootstrap_servers: Initial list of comma-seperated servers that the client will attempt to connect to
// group_id: Group name/ID that the consumer belongs to
// batch_interal: Time in ms client will wait before flushing request to brokers
impl KafkaClient {
    pub fn new(
        bootstrap_servers: &str,
        group_id: Option<&str>,
        batch_interval: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut client_config: ClientConfig = ClientConfig::new();

        client_config
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", group_id.unwrap_or("1"))
            .set("queue.buffering.max.ms", batch_interval.unwrap_or("0"));

        Ok(KafkaClient {
            client_config,
            timeout: Timeout::After(Duration::ZERO),
        })
    }

    pub fn create_producer(&self) -> Result<FutureProducer, Box<dyn std::error::Error>> {
        Ok(FutureProducer::from_config(&self.client_config)?)
    }

    pub fn create_consumer(&self) -> Result<StreamConsumer, Box<dyn std::error::Error>> {
        Ok(StreamConsumer::from_config(&self.client_config)?)
    }

    pub async fn send_to_topic(
        &self,
        future_producer: &FutureProducer,
        topic: &str,
        payload: &str,
    ) -> OwnedDeliveryResult {
        future_producer
            .send(
                FutureRecord::<(), _>::to(topic).payload(payload),
                self.timeout,
            )
            .await
    }

    pub fn subscribe_to_topic(
        &self,
        stream_consumer: &StreamConsumer,
        topic: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        stream_consumer.subscribe(&[topic])?;
        Ok(())
    }

    pub fn get_message_stream<'a>(
        &'a self,
        stream_consumer: &'a StreamConsumer,
    ) -> MessageStream<DefaultConsumerContext> {
        stream_consumer.stream()
    }
}
