use std::{sync::{atomic::AtomicUsize, Arc}, collections::HashMap, marker::PhantomData};

use futures::StreamExt;
use lapin::{options::{QueueDeleteOptions, QueueDeclareOptions, ExchangeDeclareOptions, BasicPublishOptions, QueueBindOptions, BasicConsumeOptions}, types::{FieldTable, DeliveryTag}, BasicProperties, Consumer, acker::Acker, message::Delivery};
use log::error;
use tokio::sync::{RwLock, oneshot, Mutex};

const JMS_EXCHANGE: &'static str = "JMS";
const RPC_EXCHANGE: &'static str = "JMS-RPC";

pub struct MessageQueue {
  #[allow(dead_code)]
  connection: Arc<lapin::Connection>,

  reply_queue: String,
  correlation_index: Arc<AtomicUsize>,
  correlation_map: Arc<RwLock<HashMap<String, oneshot::Sender<Delivery>>>>,
  rpc_recv_mutex: Arc<Mutex<()>>
}

impl MessageQueue {
  pub async fn new(reply_queue: &str) -> anyhow::Result<Self> {
    let rabbit_uri = std::env::var("RABBITMQ_URI").unwrap_or("amqp://localhost:5672/%2f".to_owned());
    let connection = lapin::Connection::connect(&rabbit_uri, lapin::ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    // Create the exchange if it's not already defined
    channel.exchange_declare(JMS_EXCHANGE, lapin::ExchangeKind::Topic, ExchangeDeclareOptions::default(), FieldTable::default()).await?;

    // Create the RPC exchange if it's not already defined
    channel.exchange_declare(RPC_EXCHANGE, lapin::ExchangeKind::Direct, ExchangeDeclareOptions::default(), FieldTable::default()).await?;

    // Delete the RPC reply queue if it exists, then declare it.
    channel.queue_delete(reply_queue, QueueDeleteOptions::default()).await?;
    channel.queue_declare(reply_queue, QueueDeclareOptions::default(), FieldTable::default()).await?;
    channel.queue_bind(reply_queue, RPC_EXCHANGE, reply_queue, QueueBindOptions::default(), FieldTable::default()).await?;

    Ok(MessageQueue {
      connection: Arc::new(connection),
      reply_queue: reply_queue.to_owned(),
      correlation_index: Arc::new(AtomicUsize::new(0)),
      correlation_map: Arc::new(RwLock::new(HashMap::new())),
      rpc_recv_mutex: Arc::new(Mutex::new(()))
    })
  }

  pub async fn channel(&self) -> anyhow::Result<MessageQueueChannel> {
    Ok(MessageQueueChannel {
      connection: self.connection.clone(),
      channel: self.connection.create_channel().await?,
      reply_queue: self.reply_queue.clone(),
      correlation_index: self.correlation_index.clone(),
      correlation_map: self.correlation_map.clone(),
      rpc_recv_mutex: self.rpc_recv_mutex.clone()
    })
  }
}

pub struct MessageQueueChannel {
  connection: Arc<lapin::Connection>,
  channel: lapin::Channel,

  reply_queue: String,
  correlation_index: Arc<AtomicUsize>,
  correlation_map: Arc<RwLock<HashMap<String, oneshot::Sender<Delivery>>>>,
  rpc_recv_mutex: Arc<Mutex<()>>
}

impl MessageQueueChannel {
  pub async fn clone(&self) -> anyhow::Result<Self> {
    Ok(Self {
      connection: self.connection.clone(),
      channel: self.connection.create_channel().await?,
      reply_queue: self.reply_queue.clone(),
      correlation_index: self.correlation_index.clone(),
      correlation_map: self.correlation_map.clone(),
      rpc_recv_mutex: self.rpc_recv_mutex.clone()
    })
  }

  pub async fn publish<T: serde::Serialize>(&self, topic: &str, data: T) -> anyhow::Result<()> {
    let json = serde_json::to_vec(&data).unwrap();
    self.channel.basic_publish(JMS_EXCHANGE, topic, BasicPublishOptions::default(), &json[..], BasicProperties::default()).await?.await?;
    Ok(())
  }

  async fn subscribe_internal<T: serde::de::DeserializeOwned>(&self, exchange: &str, topic: &str, queue: &str, subscriber_name: &str, backlog: bool) -> anyhow::Result<MessageQueueSubscriber<T>> {
    if !backlog {
      // Delete the queue if it already exists - we're only listening for new messages
      self.channel.queue_delete(queue, QueueDeleteOptions::default()).await?;
    }

    // Declare the bind the queue to the exchange
    let q = self.channel.queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default()).await?;
    self.channel.queue_bind(q.name().as_str(), exchange, topic, QueueBindOptions::default(), FieldTable::default()).await?;
    
    let mut options = BasicConsumeOptions::default();
    options.no_ack = true;      // Auto-ack when we receive the message
    let consumer = self.channel.basic_consume(q.name().as_str(), subscriber_name, options, FieldTable::default()).await?;
    Ok(MessageQueueSubscriber { consumer, _phantom: PhantomData })
  }

  pub async fn subscribe<T: serde::de::DeserializeOwned>(&self, topic: &str, queue: &str, subscriber_name: &str, backlog: bool) -> anyhow::Result<MessageQueueSubscriber<T>> {
    self.subscribe_internal(JMS_EXCHANGE, topic, queue, subscriber_name, backlog).await
  }

  pub async fn rpc_subscribe<T: serde::de::DeserializeOwned>(&self, topic: &str, queue: &str, subscriber_name: &str, backlog: bool) -> anyhow::Result<MessageQueueSubscriber<T>> {
    self.subscribe_internal(RPC_EXCHANGE, topic, queue, subscriber_name, backlog).await
  }

  pub async fn rpc_reply<T: serde::Serialize>(&self, incoming_props: &BasicProperties, data: T) -> anyhow::Result<()> {
    let json = serde_json::to_vec(&data).unwrap();
    match (incoming_props.correlation_id(), incoming_props.reply_to()) {
      (Some(corr), Some(reply_to)) => {
        let props = BasicProperties::default()
          .with_correlation_id(corr.clone());
        
        self.channel.basic_publish(
          RPC_EXCHANGE, 
          reply_to.as_str(),
          BasicPublishOptions::default(),
          &json[..],
          props
        ).await?;
      },
      _ => ()
    }
    Ok(())
  }

  pub async fn rpc_call<Reply: serde::de::DeserializeOwned, Request: serde::Serialize>(&self, topic: &str, data: Request) -> anyhow::Result<Reply> {
    let json = serde_json::to_vec(&data).unwrap();
    let props = BasicProperties::default()
      .with_correlation_id(
        format!("{}", self.correlation_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed)).into()
      ).with_reply_to(
        self.reply_queue.clone().into()
      );
    
    let (tx, rx) = oneshot::channel();
    let our_correlation = props.correlation_id().as_ref().unwrap().to_string();
    self.correlation_map.write().await.insert(our_correlation.clone(), tx);

    self.channel.basic_publish(
      RPC_EXCHANGE, 
      topic,
      BasicPublishOptions::default(),
      &json[..],
      props
    ).await?;

    tokio::select! {
      response = rx => { return Ok(serde_json::from_slice(&response?.data[..])?) },
      _ = self.rpc_recv_mutex.lock() => {
        // The responsibility for keeping track of RPC messages is ours
        let mut reply_consumer = self.channel.basic_consume(&self.reply_queue, &self.reply_queue, BasicConsumeOptions::default(), FieldTable::default()).await?;
        loop {
          let next = reply_consumer.next().await;
          match next {
            None => anyhow::bail!("RPC closed"),
            Some(Ok(delivery)) => {
              if let Some(correlation) = delivery.properties.correlation_id() {
                let mut map = self.correlation_map.write().await;
                let callback = map.remove(&correlation.to_string());

                if correlation.to_string() == our_correlation {
                  // We found the one belonging to us, return it
                  return Ok(serde_json::from_slice(&delivery.data[..])?);
                }

                if let Some(callback) = callback {
                  // It's not for us - maybe it's for someone else
                  callback.send(delivery).ok();
                }
              }
              
            },
            Some(Err(e)) => error!("RPC listen error: {}", e)
          }
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct TypedDelivery<T> {
  pub delivery_tag: DeliveryTag,
  pub exchange: String,
  pub routing_key: String,
  pub redelivered: bool,
  pub properties: BasicProperties,
  pub data: T,
  pub acker: Acker
}

pub struct MessageQueueSubscriber<T> {
  consumer: Consumer,
  _phantom: PhantomData<T>
}

impl<T> MessageQueueSubscriber<T> where T: serde::de::DeserializeOwned {
  pub async fn next(&mut self) -> Option<anyhow::Result<TypedDelivery<T>>> {
    let result = self.consumer.next().await?;

    match result {
      Ok(msg) => match serde_json::from_slice(&msg.data) {
        Ok(data) => Some(Ok(
          TypedDelivery {
            delivery_tag: msg.delivery_tag,
            exchange: msg.exchange.to_string(),
            routing_key: msg.routing_key.to_string(),
            redelivered: msg.redelivered,
            properties: msg.properties,
            data,
            acker: msg.acker
          }
        )),
        Err(e) => Some(Err(anyhow::anyhow!("Decode error: {}", e)))
      },
      Err(e) => Some(Err(anyhow::anyhow!("Could not receive: {}", e)))
    }
  }
}
