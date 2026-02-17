use bevy::prelude::*;

use crate::systems::{
    interfaces::GameSystemSet,
    network::{
        MyNetworkId, NetworkEntityMap, NetworkResource, handle_network_events,
        interpolate_positions, send_ping_system, setup_network, update_network_status,
    },
};

/// Client netcode systems: connection lifecycle, packet handling and interpolation.
pub struct NetcodePlugin;

impl Plugin for NetcodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkResource>()
            .init_resource::<NetworkEntityMap>()
            .init_resource::<MyNetworkId>()
            .add_systems(Startup, setup_network)
            .add_systems(
                Update,
                (
                    update_network_status,
                    handle_network_events,
                    send_ping_system,
                    interpolate_positions,
                )
                    .in_set(GameSystemSet::GameLogic),
            );
    }
}
