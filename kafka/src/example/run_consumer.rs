use futures::TryStreamExt;
use kafka::kafka_client::KafkaClient;
use rdkafka::{consumer::StreamConsumer, Message};

#[tokio::main]
async fn main() {
    let client: KafkaClient = KafkaClient::new("localhost:9094", None, None).unwrap();
    let consumer: StreamConsumer = client.create_consumer().unwrap();

    client.subscribe_to_topic(&consumer, "random").unwrap();
    client
        .get_message_stream(&consumer)
        .try_for_each(|borrowed_message| async move {
            match borrowed_message.payload_view::<str>() {
                Some(Ok(s)) => {
                    println!("Received payload: {}", s);
                }
                Some(Err(_)) => {
                    println!("Payload format is incorrect");
                }
                None => {
                    println!("Payload does not exist");
                }
            }
            Ok(())
        })
        .await
        .unwrap();
}
