// main.rs

use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::Mutex;

mod pow;
mod config;

use pow::{PowRequest, PowResponse, perform_pow};
use config::Config;

#[derive(Serialize, Deserialize)]
struct Message {
    action: String,
    payload: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    let thread_pool = Arc::new(Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .build()
            .unwrap()
    ));

    while let Ok((stream, _)) = listener.accept().await {
        let thread_pool = Arc::clone(&thread_pool);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, thread_pool).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, thread_pool: Arc<Mutex<rayon::ThreadPool>>) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        let msg = msg?;
        if msg.is_text() {
            let text = msg.to_text()?;
            let message: Message = serde_json::from_str(text)?;

            match message.action.as_str() {
                "pow_request" => {
                    let pow_request: PowRequest = serde_json::from_value(message.payload)?;
                    let thread_pool = thread_pool.lock().await;
                    let pow_response = thread_pool.install(|| perform_pow(&pow_request));
                    let response = Message {
                        action: "pow_response".to_string(),
                        payload: serde_json::to_value(pow_response)?,
                    };
                    ws_sender.send(serde_json::to_string(&response)?.into()).await?;
                }
                _ => {
                    eprintln!("Unknown action: {}", message.action);
                }
            }
        }
    }

    Ok(())
}
