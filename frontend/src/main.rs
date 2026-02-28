use dioxus::prelude::*;
use futures_util::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct VanPayload {
    van_id: String,
    lat: f64,
    lon: f64,
    timestamp: u64,
}

fn main() {
    // In Dioxus 0.5+, we no longer need dioxus_web::launch
    dioxus::launch(App);
}

// In Dioxus 0.5+, components no longer take a (cx: Scope) argument!
#[component]
fn App() -> Element {
    // SENSEI LESSON: Dioxus 0.5 uses "Signals" instead of use_ref/use_state. 
    // Signals are Copy, meaning they are incredibly easy to move into async closures!
    let mut vans = use_signal(|| HashMap::<String, VanPayload>::new());

    // Coroutines no longer require `cx` as the first argument
    use_coroutine(move |mut _rx: UnboundedReceiver<()>| async move {
        let mut ws = WebSocket::open("ws://127.0.0.1:8080/ws").unwrap();
        
        while let Some(msg) = ws.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(payload) = serde_json::from_str::<VanPayload>(&text) {
                    // We update the signal directly. This automatically triggers a UI re-render.
                    vans.write().insert(payload.van_id.clone(), payload);
                }
            }
        }
    });

    rsx! {
        div {
            style: "font-family: sans-serif; padding: 20px; background-color: #1e1e2e; color: #cdd6f4; height: 100vh;",
            h1 { "🌐 AtlasMotion Live Tracker" }
            p { "Sensei's Dioxus + Actix + Kafka Dashboard" }
            
            div {
                style: "display: flex; flex-direction: column; gap: 10px; margin-top: 20px;",
                // SENSEI LESSON: Dioxus 0.5 allows native Rust `for` loops directly inside rsx!
                // No more confusing .map() chains with missing commas.
                for van in vans.read().values() {
                    div {
                        key: "{van.van_id}",
                        style: "background-color: #313244; padding: 15px; border-radius: 8px; border-left: 5px solid #a6e3a1;",
                        strong { "{van.van_id}" }
                        span { style: "margin-left: 15px;", "🛰️ Lat: {van.lat:.5}, Lon: {van.lon:.5}" }
                    }
                }
            }
        }
    }
}