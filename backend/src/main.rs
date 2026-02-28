use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::Message as KafkaMessage;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use rand::Rng;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VanPayload {
    van_id: String,
    lat: f64,
    lon: f64,
    timestamp: u64,
}

// 1. The Producer: Now accepts a specific Van ID!
async fn run_simulator(van_id: String) {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9094")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation failed");

    // Randomize starting positions slightly so they aren't all exactly stacked
    let mut lat = 37.7749 + rand::thread_rng().gen_range(-0.05..0.05);
    let mut lon = -122.4194 + rand::thread_rng().gen_range(-0.05..0.05);

    loop {
        lat += rand::thread_rng().gen_range(-0.002..0.002);
        lon += rand::thread_rng().gen_range(-0.002..0.002);
        
        let payload = VanPayload {
            van_id: van_id.clone(),
            lat,
            lon,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        };

        let payload_json = serde_json::to_string(&payload).unwrap();
        let record = FutureRecord::to("driver-locations")
            .key(&payload.van_id)
            .payload(&payload_json);

        let _ = producer.send(record, Timeout::Never).await;
        
        // Randomize the sleep so they don't all ping Kafka at the exact same millisecond
        let sleep_time = rand::thread_rng().gen_range(1000..3000);
        tokio::time::sleep(Duration::from_millis(sleep_time)).await;
    }
}

// 2. The Consumer: Unchanged
async fn run_kafka_consumer(tx: broadcast::Sender<String>) {
    let group_id = format!("actix-broadcaster-node-{}", Uuid::new_v4());

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &group_id)
        .set("bootstrap.servers", "localhost:9094")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&["driver-locations"]).expect("Can't subscribe");
    println!("Sensei: Kafka Consumer is listening with group '{}'. Ready to broadcast...", group_id);

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Kafka error: {}", e),
            Ok(m) => {
                if let Some(payload) = m.payload() {
                    if let Ok(text) = std::str::from_utf8(payload) {
                        let _ = tx.send(text.to_string());
                    }
                }
            }
        }
    }
}

// 3. The WebSocket Handler: Unchanged
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<broadcast::Sender<String>>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, _msg_stream) = actix_ws::handle(&req, stream)?;
    let mut rx = tx.subscribe();

    tokio::task::spawn_local(async move {
        while let Ok(msg) = rx.recv().await {
            if session.text(msg).await.is_err() {
                break;
            }
        }
    });

    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, _) = broadcast::channel::<String>(1000); // Increased channel buffer for more traffic
    
    // 🚀 SENSEI LESSON: Scale the Fleet!
    // We spawn 100 independent asynchronous workers. Tokio will distribute these
    // seamlessly across your CPU cores.
    for i in 1..=100 {
        let van_id = format!("VAN-{:03}", i);
        tokio::spawn(run_simulator(van_id));
    }

    tokio::spawn(run_kafka_consumer(tx.clone()));

    let app_data = web::Data::new(tx);

    println!("Sensei: Actix Server running at http://127.0.0.1:8080 with 100 Vans!");
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}