pub use lapin::*;
use lapin::{options::{ExchangeDeclareOptions, QueueDeleteOptions, QueueDeclareOptions, QueueBindOptions, BasicConsumeOptions, BasicPublishOptions}, types::FieldTable, Consumer, Queue, BasicProperties};

pub async fn rabbit_connect() -> anyhow::Result<lapin::Connection> {
  let rabbit_uri = std::env::var("RABBITMQ_URI").unwrap_or("amqp://rabbitmq:5672/%2f".to_owned());
  
  let connection = lapin::Connection::connect(&rabbit_uri, lapin::ConnectionProperties::default()).await?;
  let channel = connection.create_channel().await?;

  // Create the exchange if it's not already defined
  channel.exchange_declare("JMS", lapin::ExchangeKind::Topic, ExchangeDeclareOptions::default(), FieldTable::default()).await?;

  Ok(connection)
}

pub async fn rabbit_subscribe(channel: &lapin::Channel, topics: &[&str], queue_name: &str, consumer_name: &str, gen_unique: bool, delete_old: bool) -> anyhow::Result<(Queue, Consumer)> {
  let mut queue_name = queue_name.to_owned();
  if gen_unique {
    queue_name += &("#".to_owned() + gethostname::gethostname().to_str().unwrap());
  }

  if delete_old {
    channel.queue_delete(&queue_name, QueueDeleteOptions::default()).await?;
  }

  let queue = channel.queue_declare(&queue_name, QueueDeclareOptions::default(), FieldTable::default()).await?;
  for topic in topics {
    channel.queue_bind(&queue_name, "JMS", topic, QueueBindOptions::default(), FieldTable::default()).await?;
  }

  let consumer = channel.basic_consume(&queue_name, consumer_name, BasicConsumeOptions::default(), FieldTable::default()).await?;

  Ok((queue, consumer))
}

pub async fn rabbit_publish<T: serde::Serialize>(channel: &lapin::Channel, topic: &str, data: T) -> anyhow::Result<()> {
  let json = serde_json::to_vec(&data).unwrap();
  channel.basic_publish("JMS", topic, BasicPublishOptions::default(), &json[..], BasicProperties::default()).await?.await?;
  Ok(())
}
