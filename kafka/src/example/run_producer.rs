use kafka::kafka_client::KafkaClient;
use rdkafka::producer::FutureProducer;

#[tokio::main]
async fn main() {
    let client: KafkaClient = KafkaClient::new("localhost:9094", None, None).unwrap();
    let producer: FutureProducer = client.create_producer().unwrap();

    client
        .send_to_topic(&producer, "random", "randomly generated payload")
        .await
        .unwrap();
}
