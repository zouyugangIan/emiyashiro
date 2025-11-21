use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;
use s__emiyashiro::protocol::{GamePacket, PlayerAction};
use futures_lite::future;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("Starting G-Engine Server...");

    // Channel for sending actions from Network -> Bevy
    let (action_tx, action_rx) = mpsc::unbounded_channel::<(u64, PlayerAction)>();
    // Channel for broadcasting packets from Bevy -> Network
    let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<GamePacket>();

    // Initialize Database
    let database = s__emiyashiro::database::Database::new().await.expect("Failed to connect to database");
    let pool = database.pool.clone();

    // Spawn Save Worker
    tokio::spawn(async move {
        s__emiyashiro::systems::save_worker::run_save_worker(pool).await;
    });

    // Shared state for connected clients (id -> sender)
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let clients_clone = clients.clone();

    // Spawn Network Task
    tokio::spawn(async move {
        let addr = "127.0.0.1:8080";
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        println!("WebSocket server listening on: {}", addr);

        let mut client_id_counter = 0;

        while let Ok((stream, _)) = listener.accept().await {
            client_id_counter += 1;
            let client_id = client_id_counter;
            let clients_inner = clients_clone.clone();
            let action_tx_inner = action_tx.clone();

            tokio::spawn(async move {
                handle_connection(stream, client_id, clients_inner, action_tx_inner).await;
            });
        }
    });

    // Spawn Broadcast Task
    let clients_broadcast = clients.clone();
    tokio::spawn(async move {
        while let Some(packet) = broadcast_rx.recv().await {
            let msg = bincode::serialize(&packet).unwrap();
            let mut clients_guard = clients_broadcast.lock().unwrap();
            // Remove disconnected clients
            clients_guard.retain(|_, tx: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Message>| {
                let _ = tx.start_send_unpin(tokio_tungstenite::tungstenite::Message::Binary(msg.clone()));
                true // We can't easily detect disconnect here without await, so we rely on the read loop to clean up
            });
            // Flush all
            for tx in clients_guard.values_mut() {
                 let _ = tx.poll_ready_unpin(&mut std::task::Context::from_waker(&futures_util::task::noop_waker()));
                 let _ = tx.start_send_unpin(tokio_tungstenite::tungstenite::Message::Binary(msg.clone()));
            }
        }
    });

    let mut app = App::new();

    // Server runs headless
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))));
    app.add_plugins(s__emiyashiro::database::redis::RedisPlugin);
    
    // Insert resources to communicate with network
    app.insert_resource(NetworkChannels {
        action_rx: Arc::new(Mutex::new(action_rx)),
        broadcast_tx,
    });

    app.init_resource::<ClientEntityMap>();
    
    app.add_systems(Startup, setup_bots);

    app.add_systems(Update, (
        process_network_events, 
        broadcast_snapshot_system, 
        s__emiyashiro::systems::sync_redis::sync_transform_to_redis, 
        server_physics_system,
        s__emiyashiro::systems::ai::bot_control_system
    ));

    app.run();
}

async fn handle_connection(
    stream: TcpStream,
    client_id: u64,
    clients: Arc<Mutex<HashMap<u64, futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Message>>>>,
    action_tx: mpsc::UnboundedSender<(u64, PlayerAction)>,
) {
    let ws_stream = accept_async(stream).await.expect("Error during handshake");
    println!("New client connected: {}", client_id);

    let (tx, mut rx) = ws_stream.split();

    // Register client
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.insert(client_id, tx);
    }

    // Send Welcome
    let welcome = GamePacket::Welcome { id: client_id, message: "Connected to G-Engine Server".to_string() };
    let _ = action_tx.send((client_id, PlayerAction::Ping(0))); // Simulate initial ping

    while let Some(msg) = rx.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Binary(bin)) => {
                if let Ok(action) = bincode::deserialize::<PlayerAction>(&bin) {
                    let _ = action_tx.send((client_id, action));
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
            _ => {}
        }
    }

    // Cleanup
    println!("Client disconnected: {}", client_id);
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.remove(&client_id);
}

#[derive(Resource)]
struct NetworkChannels {
    action_rx: Arc<Mutex<mpsc::UnboundedReceiver<(u64, PlayerAction)>>>,
    broadcast_tx: mpsc::UnboundedSender<GamePacket>,
}

use s__emiyashiro::components::physics::Velocity;
use s__emiyashiro::components::player::{Player, PlayerInputState};
use s__emiyashiro::components::network::NetworkId;
use s__emiyashiro::components::ai::BotController;

#[derive(Resource, Default)]
struct ClientEntityMap(HashMap<u64, Entity>);

fn setup_bots(mut commands: Commands) {
    println!("Spawning Bot...");
    commands.spawn((
        Transform::from_xyz(100.0, 0.0, 0.0),
        Velocity::zero(),
        Player,
        PlayerInputState::default(),
        BotController::default(),
        NetworkId(9999),
    ));
}

fn process_network_events(
    mut commands: Commands,
    channels: Res<NetworkChannels>,
    mut client_map: ResMut<ClientEntityMap>,
    mut query: Query<&mut PlayerInputState>,
) {
    let mut rx = channels.action_rx.lock().unwrap();
    while let Ok((client_id, action)) = rx.try_recv() {
        // Ensure player entity exists
        let entity = *client_map.0.entry(client_id).or_insert_with(|| {
            println!("Spawning player for client {}", client_id);
            commands.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity::zero(),
                Player,
                NetworkId(client_id),
                PlayerInputState::default(),
            )).id()
        });

        match action {
            PlayerAction::Ping(id) => {
                let _ = channels.broadcast_tx.send(GamePacket::Pong(id));
            }
            PlayerAction::Move { x, y } => {
                if let Ok(mut input) = query.get_mut(entity) {
                    input.move_x = x;
                    input.move_y = y;
                }
            }
            PlayerAction::Jump => {
                if let Ok(mut input) = query.get_mut(entity) {
                    input.jump_pressed = true;
                }
            }
            _ => {}
        }
    }
}

fn server_physics_system(
    mut query: Query<(&mut Transform, &mut Velocity, &mut PlayerInputState)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    for (mut transform, mut velocity, mut input) in query.iter_mut() {
        // Apply movement
        velocity.x = input.move_x * 200.0;
        
        // Apply jump
        if input.jump_pressed {
            velocity.y = 500.0;
            input.jump_pressed = false; // Reset jump
        }

        // Apply gravity
        velocity.y -= 980.0 * delta_time;

        // Apply velocity
        transform.translation.x += velocity.x * delta_time;
        transform.translation.y += velocity.y * delta_time;

        // Ground collision
        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

fn broadcast_snapshot_system(
    channels: Res<NetworkChannels>,
    query: Query<(&Transform, &Velocity, &NetworkId)>,
) {
    let mut players = Vec::new();
    for (transform, velocity, net_id) in query.iter() {
        players.push(s__emiyashiro::protocol::PlayerState {
            id: net_id.0,
            position: transform.translation,
            velocity: Vec3::new(velocity.x, velocity.y, 0.0),
            facing_right: velocity.x >= 0.0,
            animation_state: "Idle".to_string(), // Placeholder
        });
    }

    if !players.is_empty() {
        let snapshot = GamePacket::WorldSnapshot {
            tick: 0, // TODO: Implement tick counter
            players,
        };
        let _ = channels.broadcast_tx.send(snapshot);
    }
}
