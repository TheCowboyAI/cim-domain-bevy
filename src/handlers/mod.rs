//! Bevy Domain Handlers
//!
//! In ECS, command handlers are systems that process commands and emit events.
//! These systems enforce business rules and maintain aggregate invariants.

use crate::aggregate::*;
use crate::commands::*;
use crate::events::*;
use crate::value_objects::*;
use bevy::prelude::*;

/// System that handles CreateVisualNode commands
pub fn handle_create_visual_node(
    mut commands: Commands,
    mut create_events: EventReader<CreateVisualNode>,
    mut created_events: EventWriter<VisualNodeCreated>,
) {
    for event in create_events.read() {
        // Create the aggregate entity with its components
        let entity = commands
            .spawn(VisualNodeAggregate::new(
                event.node_id.clone(),
                event.position.clone(),
            ))
            .id();

        // Emit domain event
        created_events.send(VisualNodeCreated {
            entity,
            node_id: event.node_id.clone(),
            position: event.position.clone(),
        });
    }
}

/// System that handles MoveNode commands
pub fn handle_move_node(
    mut move_events: EventReader<MoveNode>,
    mut moved_events: EventWriter<NodeMoved>,
    mut query: Query<(Entity, &NodeId, &mut Position)>,
) {
    for event in move_events.read() {
        // Find the node entity
        for (entity, node_id, mut position) in query.iter_mut() {
            if node_id == &event.node_id {
                let old_position = position.clone();

                // Apply business rule: validate position is within bounds
                if event.new_position.is_valid() {
                    *position = event.new_position.clone();

                    // Emit domain event
                    moved_events.send(NodeMoved {
                        entity,
                        node_id: node_id.clone(),
                        old_position,
                        new_position: event.new_position.clone(),
                    });
                }
                break;
            }
        }
    }
}

/// System that handles DeleteVisualNode commands
pub fn handle_delete_visual_node(
    mut commands: Commands,
    mut delete_events: EventReader<DeleteVisualNode>,
    mut deleted_events: EventWriter<VisualNodeDeleted>,
    query: Query<(Entity, &NodeId, &Position)>,
) {
    for event in delete_events.read() {
        // Find and remove the node entity
        for (entity, node_id, position) in query.iter() {
            if node_id == &event.node_id {
                // Store final state before deletion
                let final_position = position.clone();

                // Remove the entity (aggregate)
                commands.entity(entity).despawn();

                // Emit domain event
                deleted_events.send(VisualNodeDeleted {
                    node_id: node_id.clone(),
                    final_position,
                });
                break;
            }
        }
    }
}

/// System that handles SelectNode commands
pub fn handle_select_node(
    mut select_events: EventReader<SelectNode>,
    mut selected_events: EventWriter<NodeSelected>,
    mut deselected_events: EventWriter<NodeDeselected>,
    mut query: Query<(Entity, &NodeId, &mut InteractionState)>,
) {
    for event in select_events.read() {
        // Handle multi-select logic
        if !event.multi_select {
            // Deselect all other nodes
            for (entity, node_id, mut state) in query.iter_mut() {
                if state.is_selected && node_id != &event.node_id {
                    state.is_selected = false;
                    deselected_events.send(NodeDeselected {
                        entity,
                        node_id: node_id.clone(),
                    });
                }
            }
        }

        // Select the target node
        for (entity, node_id, mut state) in query.iter_mut() {
            if node_id == &event.node_id && !state.is_selected {
                state.is_selected = true;
                selected_events.send(NodeSelected {
                    entity,
                    node_id: node_id.clone(),
                });
                break;
            }
        }
    }
}

/// Plugin that registers all command handlers
pub struct CommandHandlerPlugin;

impl Plugin for CommandHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_create_visual_node,
                handle_move_node,
                handle_delete_visual_node,
                handle_select_node,
            ),
        );
    }
}
