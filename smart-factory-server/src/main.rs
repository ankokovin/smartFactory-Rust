use std::{env, io::Error};

use futures_util::{SinkExt, StreamExt};
use smart_factory_environment::greet_message;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    loop {
        let msg = read.next().await;
        match msg {
            Some(Ok(message)) => {
                let message = message;
                if let tungstenite::Message::Text(message) = message {
                    let result = write
                        .send(tungstenite::Message::Text(greet_message(&message)))
                        .await;
                    if result.is_err() {
                        println!(
                            "Error while sending a message: {:?}",
                            result.unwrap_err()
                        );
                    }
                }
            }
            _ => break,
        }
    }
}
