use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use s_emiyashiro::plugins::server::{NetworkChannels, ServerRuntimePlugin};
use s_emiyashiro::protocol::{GamePacket, PlayerAction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;

type WsMessage = tokio_tungstenite::tungstenite::Message;
type ClientMessageSender = mpsc::UnboundedSender<WsMessage>;
type ClientSenderMap = HashMap<u64, ClientMessageSender>;
type SharedClients = Arc<Mutex<ClientSenderMap>>;

#[tokio::main]
async fn main() {
    info!("Starting G-Engine Server...");

    let (action_tx, action_rx) = mpsc::unbounded_channel::<(u64, PlayerAction)>();
    let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<GamePacket>();

    #[cfg(feature = "server")]
    {
        let database = s_emiyashiro::database::Database::new()
            .await
            .expect("Failed to connect to database");
        let pool = database.pool.clone();

        tokio::spawn(async move {
            s_emiyashiro::systems::save_worker::run_save_worker(pool).await;
        });
    }

    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));
    let clients_clone = clients.clone();

    tokio::spawn(async move {
        let addr = "127.0.0.1:8080";
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        info!("WebSocket server listening on: {}", addr);

        let mut client_id_counter: u64 = 0;

        while let Ok((stream, _)) = listener.accept().await {
            client_id_counter = client_id_counter.wrapping_add(1);
            let client_id = client_id_counter;
            let clients_inner = clients_clone.clone();
            let action_tx_inner = action_tx.clone();

            tokio::spawn(async move {
                handle_connection(stream, client_id, clients_inner, action_tx_inner).await;
            });
        }
    });

    let clients_broadcast = clients.clone();
    tokio::spawn(async move {
        while let Some(packet) = broadcast_rx.recv().await {
            let binary = match bincode::serialize(&packet) {
                Ok(bytes) => bytes,
                Err(error) => {
                    warn!("Failed to serialize packet: {}", error);
                    continue;
                }
            };

            let mut stale_clients = Vec::new();
            {
                let clients_guard = match clients_broadcast.lock() {
                    Ok(guard) => guard,
                    Err(_) => continue,
                };

                for (client_id, sender) in clients_guard.iter() {
                    if sender.send(WsMessage::Binary(binary.clone())).is_err() {
                        stale_clients.push(*client_id);
                    }
                }
            }

            if !stale_clients.is_empty()
                && let Ok(mut clients_guard) = clients_broadcast.lock()
            {
                for client_id in stale_clients {
                    clients_guard.remove(&client_id);
                }
            }
        }
    });

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    );

    #[cfg(feature = "server")]
    app.add_plugins(s_emiyashiro::database::redis::RedisPlugin);

    app.add_plugins(ServerRuntimePlugin {
        channels: NetworkChannels {
            action_rx: Arc::new(Mutex::new(action_rx)),
            broadcast_tx,
        },
    });

    app.run();
}

async fn handle_connection(
    stream: TcpStream,
    client_id: u64,
    clients: SharedClients,
    action_tx: mpsc::UnboundedSender<(u64, PlayerAction)>,
) {
    let ws_stream = accept_async(stream).await.expect("Error during handshake");
    info!("New client connected: {}", client_id);

    let (mut write, mut read) = ws_stream.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<WsMessage>();

    let writer_handle = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let welcome = GamePacket::Welcome {
        id: client_id,
        message: "Connected to G-Engine Server".to_string(),
    };
    match bincode::serialize(&welcome) {
        Ok(binary) => {
            let _ = out_tx.send(WsMessage::Binary(binary));
        }
        Err(error) => {
            warn!("Failed to serialize welcome packet: {}", error);
        }
    }

    if let Ok(mut clients_guard) = clients.lock() {
        clients_guard.insert(client_id, out_tx.clone());
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(WsMessage::Binary(bin)) => {
                if let Ok(action) = bincode::deserialize::<PlayerAction>(&bin) {
                    let _ = action_tx.send((client_id, action));
                }
            }
            Ok(WsMessage::Close(_)) => break,
            _ => {}
        }
    }

    info!("Client disconnected: {}", client_id);
    if let Ok(mut clients_guard) = clients.lock() {
        clients_guard.remove(&client_id);
    }

    drop(out_tx);
    let _ = writer_handle.await;
}
