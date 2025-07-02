//! Bridge between domain events and Bevy ECS

use bevy::prelude::*;
use crate::events::VisualizationCommand;
use crossbeam_channel::{Receiver, Sender, bounded};

/// Error types for bridge operations
#[derive(Debug, Clone)]
pub enum BridgeError {
    /// Channel is disconnected
    ChannelDisconnected,
    /// Channel is full
    ChannelFull,
}

/// Bridge between async domain layer and sync Bevy ECS
#[derive(Resource)]
pub struct AsyncSyncBridge {
    /// Channel for visualization commands (domain → visual)
    domain_to_bevy: (Sender<VisualizationCommand>, Receiver<VisualizationCommand>),
    /// Channel for visualization commands (visual → domain)
    bevy_to_domain: (Sender<VisualizationCommand>, Receiver<VisualizationCommand>),
}

impl AsyncSyncBridge {
    /// Create a new bridge with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (domain_tx, domain_rx) = bounded(capacity);
        let (bevy_tx, bevy_rx) = bounded(capacity);

        Self {
            domain_to_bevy: (domain_tx, domain_rx),
            bevy_to_domain: (bevy_tx, bevy_rx),
        }
    }

    /// Get sender for visualization commands (used by async domain layer)
    pub fn domain_sender(&self) -> Sender<VisualizationCommand> {
        self.domain_to_bevy.0.clone()
    }

    /// Get receiver for visualization commands (used by async domain layer)
    pub fn command_receiver(&self) -> Receiver<VisualizationCommand> {
        self.bevy_to_domain.1.clone()
    }

    /// Send command from Bevy to domain
    pub fn send_command(&self, command: VisualizationCommand) -> Result<(), BridgeError> {
        self.bevy_to_domain.0.send(command)
            .map_err(|_| BridgeError::ChannelDisconnected)
    }

    /// Receive visualization commands from domain (non-blocking)
    pub fn receive_domain_events(&self) -> Vec<VisualizationCommand> {
        let mut events = Vec::new();
        while let Ok(event) = self.domain_to_bevy.1.try_recv() {
            events.push(event);
        }
        events
    }

    /// Send a visualization command to the domain layer
    pub fn send_domain_event(&self, event: VisualizationCommand) {
        println!("🌉 Bridge: Sending visualization command to channel: {event:?}");
        if let Err(e) = self.domain_sender().send(event) {
            eprintln!("Failed to send visualization command: {e:?}");
        }
    }
}

/// System that processes visualization commands from the async channel
pub fn process_domain_events(
    bridge: Res<AsyncSyncBridge>,
    mut viz_commands: EventWriter<VisualizationCommand>,
) {
    let events = bridge.receive_domain_events();
    if !events.is_empty() {
        println!("🌉 Bridge: Received {} visualization commands from channel", events.len());
    }
    for event in events {
        println!("  📥 Processing: {event:?}");
        viz_commands.write(event);
    }
}

/// System that sends visualization commands to the domain
pub fn send_visualization_commands(
    mut viz_commands: EventReader<VisualizationCommand>,
    bridge: Res<AsyncSyncBridge>,
) {
    for cmd in viz_commands.read() {
        println!("🌉 Bridge: Sending visualization command: {cmd:?}");
        if let Err(e) = bridge.send_command(cmd.clone()) {
            eprintln!("Failed to send visualization command: {e:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_contextgraph::NodeId;

    #[test]
    fn test_bridge_creation() {
        let bridge = AsyncSyncBridge::new(100);

        // Test sending visualization command
        let event = VisualizationCommand::CreateNode(crate::events::CreateNodeVisual {
            node_id: NodeId::new(),
            position: Vec3::ZERO,
            label: "Test".to_string(),
        });

        bridge.domain_sender().send(event).unwrap();

        // Receive events
        let received = bridge.receive_domain_events();
        assert_eq!(received.len(), 1);
    }
}
