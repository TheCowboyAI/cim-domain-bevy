//! NATS-based bridge for isomorphic component synchronization between DDD and Bevy ECS
//!
//! This module provides the Bevy side of the isomorphic component architecture,
//! receiving component events from NATS and applying them to Bevy entities.

use bevy::prelude::*;
use cim_domain::{ComponentEvent, EcsComponentData};
use async_nats::Client;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Resource for managing NATS component synchronization in Bevy
#[derive(Resource)]
pub struct NatsComponentBridge {
    /// Channel to receive component events from NATS
    event_receiver: mpsc::UnboundedReceiver<ComponentEvent>,
    /// Handle to the subscription task
    _subscription_handle: tokio::task::JoinHandle<()>,
}

impl NatsComponentBridge {
    /// Create a new NATS component bridge
    pub async fn new(nats_client: Arc<Client>) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Subscribe to component events
        let mut subscription = nats_client
            .subscribe("cim.component.>")
            .await?;
        
        // Spawn task to forward events to channel
        let handle = tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                if let Ok(event) = serde_json::from_slice::<ComponentEvent>(&message.payload) {
                    let _ = tx.send(event);
                }
            }
        });
        
        Ok(Self {
            event_receiver: rx,
            _subscription_handle: handle,
        })
    }
    
    /// Receive pending component events (non-blocking)
    pub fn receive_events(&mut self) -> Vec<ComponentEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

/// Component to mark entities that are synchronized via NATS
#[derive(Component)]
pub struct NatsSyncedEntity {
    /// The UUID that maps to the DDD entity
    pub entity_id: Uuid,
}

/// Marker component for entities that need component updates
#[derive(Component)]
pub struct PendingComponentUpdate {
    /// The component data to apply
    pub component_data: EcsComponentData,
}

/// System to process component events from NATS
pub fn process_nats_component_events(
    mut bridge: ResMut<NatsComponentBridge>,
    mut commands: Commands,
    query: Query<(Entity, &NatsSyncedEntity)>,
) {
    let events = bridge.receive_events();
    
    for event in events {
        match event {
            ComponentEvent::Added { entity_id, component_data } |
            ComponentEvent::Updated { entity_id, component_data } => {
                // Find the Bevy entity with this UUID
                let bevy_entity = query
                    .iter()
                    .find(|(_, synced)| synced.entity_id == entity_id)
                    .map(|(entity, _)| entity);
                
                if let Some(entity) = bevy_entity {
                    // Mark entity for component update
                    commands.entity(entity).insert(PendingComponentUpdate {
                        component_data,
                    });
                } else {
                    // Create new entity with sync marker
                    commands.spawn((
                        NatsSyncedEntity { entity_id },
                        PendingComponentUpdate { component_data },
                    ));
                }
            }
            
            ComponentEvent::Removed { entity_id, component_type } => {
                // Handle component removal if needed
                if let Some((entity, _)) = query
                    .iter()
                    .find(|(_, synced)| synced.entity_id == entity_id)
                {
                    info!("Component {} removed from entity {:?}", component_type, entity);
                    // Component-specific removal logic would go here
                }
            }
        }
    }
}

/// System to apply pending component updates
/// This is where you'd map EcsComponentData to actual Bevy components
pub fn apply_component_updates(
    mut commands: Commands,
    query: Query<(Entity, &PendingComponentUpdate), With<NatsSyncedEntity>>,
) {
    for (entity, pending) in query.iter() {
        // Map component types to Bevy components
        match pending.component_data.component_type.as_str() {
            "Position3D" => {
                if let Ok(pos) = serde_json::from_value::<Position3D>(pending.component_data.data.clone()) {
                    commands.entity(entity).insert(Transform::from_translation(
                        Vec3::new(pos.x, pos.y, pos.z)
                    ));
                }
            }
            "Label" => {
                if let Ok(label) = serde_json::from_value::<LabelData>(pending.component_data.data.clone()) {
                    commands.entity(entity).insert(Name::new(label.text));
                }
            }
            "WorkflowStateComponent" => {
                if let Ok(state) = serde_json::from_value::<WorkflowState>(pending.component_data.data.clone()) {
                    // Apply workflow-specific visualization
                    info!("Workflow state updated: {:?}", state);
                    // You would add workflow visualization components here
                }
            }
            _ => {
                warn!("Unknown component type: {}", pending.component_data.component_type);
            }
        }
        
        // Remove the pending update marker
        commands.entity(entity).remove::<PendingComponentUpdate>();
    }
}

/// Plugin to add NATS component synchronization to Bevy
pub struct NatsComponentPlugin {
    nats_client: Arc<Client>,
}

impl NatsComponentPlugin {
    /// Create a new plugin with the given NATS client
    pub fn new(nats_client: Arc<Client>) -> Self {
        Self { nats_client }
    }
}

impl Plugin for NatsComponentPlugin {
    fn build(&self, app: &mut App) {
        // Create the bridge in a blocking context
        let bridge = futures::executor::block_on(
            NatsComponentBridge::new(self.nats_client.clone())
        ).expect("Failed to create NATS component bridge");
        
        app.insert_resource(bridge)
            .add_systems(Update, (
                process_nats_component_events,
                apply_component_updates,
            ).chain());
    }
}

// Helper structs for deserialization
#[derive(serde::Deserialize)]
struct Position3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(serde::Deserialize)]
struct LabelData {
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct WorkflowState {
    state: String,
    definition_id: String,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nats_synced_entity() {
        let entity_id = Uuid::new_v4();
        let synced = NatsSyncedEntity { entity_id };
        assert_eq!(synced.entity_id, entity_id);
    }
}