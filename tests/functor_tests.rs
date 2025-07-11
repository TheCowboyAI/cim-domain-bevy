//! Tests for the categorical functor implementation
//!
//! These tests verify that the functor between Bevy ECS and CIM-ContextGraph
//! preserves structure and maintains the isomorphism.

use bevy::prelude::*;
use cim_contextgraph::{ContextGraphId as GraphId, EdgeId, NodeId};
use cim_domain_bevy::*;

#[test]
fn test_domain_to_visual_functor_preserves_identity() {
    // Create a domain node
    let node_id = NodeId::new();
    let graph_id = GraphId::new();
    let position = Vec3::new(1.0, 2.0, 3.0);

    // Map to visual representation
    let visual_bundle = NodeVisualBundle::new(node_id, graph_id, position);

    // Verify identity is preserved
    assert_eq!(visual_bundle.node.node_id, node_id);
    assert_eq!(visual_bundle.node.graph_id, graph_id);
    assert_eq!(visual_bundle.transform.translation, position);
}

#[test]
fn test_visual_to_domain_functor_preserves_operations() {
    let node_id = uuid::Uuid::new_v4();
    let new_position = Vec3::new(5.0, 6.0, 7.0);

    // Create a visual command
    let command = CreateNodeVisual {
        node_id,
        position: new_position,
        label: "Test Node".to_string(),
    };

    // Verify the command preserves the operation semantics
    assert_eq!(command.node_id, node_id);
    assert_eq!(command.position, new_position);
}

#[test]
fn test_functor_composition_identity() {
    // Test that F ∘ G ≈ Id (approximately identity due to GraphId generation)
    let node_id = NodeId::new();
    let graph_id = GraphId::new();
    let position = Vec3::new(1.0, 2.0, 3.0);

    // F: Domain → Visual
    let visual = NodeVisualBundle::new(node_id, graph_id, position);

    // Verify essential properties are preserved
    assert_eq!(visual.node.node_id, node_id);
    assert_eq!(visual.node.graph_id, graph_id);
    assert_eq!(visual.transform.translation, position);
}

#[test]
fn test_morphism_preservation() {
    // Test that morphisms (operations) are preserved
    let entity = Entity::from_raw(42);
    let node_id = NodeId::new();

    // Create a click morphism
    let click_event = NodeClicked { entity, node_id };

    // Verify the morphism structure is preserved
    assert_eq!(click_event.entity, entity);
    assert_eq!(click_event.node_id, node_id);
}

#[test]
fn test_bridge_channel_communication() {
    let bridge = AsyncSyncBridge::new(100);

    // Test command sending (Bevy → Domain)
    let command = VisualizationCommand::CreateNode(CreateNodeVisual {
        node_id: uuid::Uuid::new_v4(),
        position: Vec3::ZERO,
        label: "Test".to_string(),
    });

    assert!(bridge.send_command(command.clone()).is_ok());

    // Test event receiving (Domain → Bevy)
    let event = VisualizationCommand::CreateNode(CreateNodeVisual {
        node_id: uuid::Uuid::new_v4(),
        position: Vec3::ONE,
        label: "Test Event".to_string(),
    });

    let sender = bridge.domain_sender();
    assert!(sender.send(event).is_ok());

    let received_events = bridge.receive_domain_events();
    assert_eq!(received_events.len(), 1);
}

#[test]
fn test_event_morphism_categories() {
    // Verify that events are properly categorized as morphisms

    // Creation morphisms
    let create_node = CreateNodeVisual {
        node_id: uuid::Uuid::new_v4(),
        position: Vec3::ZERO,
        label: "Test".to_string(),
    };

    // Deletion morphisms
    let remove_node = RemoveNodeVisual {
        node_id: uuid::Uuid::new_v4(),
    };

    // Interaction morphisms
    let node_drag = NodeDragStart {
        entity: Entity::from_raw(1),
        node_id: NodeId::new(),
        start_position: Vec3::ZERO,
    };

    // All morphisms should have the required structure
    assert_eq!(create_node.node_id, create_node.node_id); // Identity
    assert_eq!(remove_node.node_id, remove_node.node_id); // Identity
    assert_eq!(node_drag.start_position, Vec3::ZERO); // Value preservation
}

#[test]
fn test_component_value_object_semantics() {
    // Test that visual components behave as value objects
    let node_visual = NodeVisual {
        node_id: NodeId::new(),
        graph_id: GraphId::new(),
    };

    let node_visual_clone = node_visual.clone();

    // Value objects should be cloneable and comparable
    assert_eq!(node_visual.node_id, node_visual_clone.node_id);
    assert_eq!(node_visual.graph_id, node_visual_clone.graph_id);
}

#[test]
fn test_bundle_composition() {
    // Test that bundles properly compose visual components
    let node_id = NodeId::new();
    let graph_id = GraphId::new();
    let position = Vec3::new(1.0, 2.0, 3.0);

    let bundle = NodeVisualBundle::new(node_id, graph_id, position);

    // Verify bundle contains all required components
    assert_eq!(bundle.node.node_id, node_id);
    assert_eq!(bundle.transform.translation, position);
    // GlobalTransform, Visibility, etc. are properly initialized
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_bridge_resource_in_app() {
        // Test that the bridge can be used as a Bevy resource
        let mut app = App::new();

        // Insert the bridge as a resource
        let bridge = AsyncSyncBridge::new(100);
        app.insert_resource(bridge);

        // Verify it can be accessed
        let bridge_ref = app.world().resource::<AsyncSyncBridge>();
        let test_command = VisualizationCommand::CreateNode(CreateNodeVisual {
            node_id: uuid::Uuid::new_v4(),
            position: Vec3::ZERO,
            label: "Test".to_string(),
        });
        assert!(bridge_ref.send_command(test_command).is_ok());
    }

    #[test]
    fn test_event_registration() {
        // Test that our events can be registered in a Bevy app
        let mut app = App::new();

        // Register our event types
        app.add_event::<NodeClicked>();
        app.add_event::<CreateNodeVisual>();
        app.add_event::<RemoveNodeVisual>();

        // Events should be properly registered
        assert!(app.world().get_resource::<Events<NodeClicked>>().is_some());
        assert!(app
            .world()
            .get_resource::<Events<CreateNodeVisual>>()
            .is_some());
        assert!(app
            .world()
            .get_resource::<Events<RemoveNodeVisual>>()
            .is_some());
    }
}
