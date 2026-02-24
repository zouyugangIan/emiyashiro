use bevy::prelude::*;

use crate::systems::{
    interfaces::GameSystemSet,
    network::{
        MyNetworkId, NetworkConfig, NetworkEntityMap, NetworkReconnectState, NetworkResource,
        auto_reconnect_network, handle_network_events, interpolate_positions,
        send_heartbeat_ping_system, send_ping_system, setup_network, update_network_status,
    },
};

/// Client netcode systems: connection lifecycle, packet handling and interpolation.
pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkResource>()
            .init_resource::<NetworkConfig>()
            .init_resource::<NetworkReconnectState>()
            .init_resource::<NetworkEntityMap>()
            .init_resource::<MyNetworkId>()
            .add_systems(Startup, setup_network)
            .add_systems(
                Update,
                (
                    update_network_status,
                    auto_reconnect_network,
                    handle_network_events,
                    send_ping_system,
                    send_heartbeat_ping_system,
                    interpolate_positions,
                )
                    .chain()
                    .in_set(GameSystemSet::GameLogic),
            );
    }
}
