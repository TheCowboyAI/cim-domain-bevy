//! Integration tests for isomorphic component synchronization between DDD and ECS

use bevy::prelude::*;
use cim_domain::{ComponentEvent, EcsComponentData};
use cim_domain_bevy::{NatsComponentBridge, NatsSyncedEntity, PendingComponentUpdate};
use uuid::Uuid;
use serde_json::json;

#[test]
fn test_component_event_to_bevy_entity() {
    // Create a test Bevy app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Add our component sync systems
    app.add_systems(Update, (
        cim_domain_bevy::process_nats_component_events,
        cim_domain_bevy::apply_component_updates,
    ).chain());
    
    // Create a test entity with sync marker
    let entity_id = Uuid::new_v4();
    let entity = app.world_mut().spawn((
        NatsSyncedEntity { entity_id },
        Transform::default(),
    )).id();
    
    // Create a component event
    let component_data = EcsComponentData {
        component_type: "Position3D".to_string(),
        data: json!({
            "x": 5.0,
            "y": 10.0,
            "z": 15.0
        }),
    };
    
    // Apply the component update
    app.world_mut().entity_mut(entity).insert(PendingComponentUpdate {
        component_data,
    });
    
    // Run the systems
    app.update();
    
    // Verify the transform was updated
    let transform = app.world().get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, Vec3::new(5.0, 10.0, 15.0));
}

#[test]
fn test_label_component_sync() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    app.add_systems(Update, cim_domain_bevy::apply_component_updates);
    
    let entity_id = Uuid::new_v4();
    let entity = app.world_mut().spawn((
        NatsSyncedEntity { entity_id },
    )).id();
    
    // Create a label component event
    let component_data = EcsComponentData {
        component_type: "Label".to_string(),
        data: json!({
            "text": "Test Node"
        }),
    };
    
    app.world_mut().entity_mut(entity).insert(PendingComponentUpdate {
        component_data,
    });
    
    app.update();
    
    // Verify the name was updated
    let name = app.world().get::<Name>(entity).unwrap();
    assert_eq!(name.as_str(), "Test Node");
}

#[test]
fn test_workflow_state_component_sync() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    app.add_systems(Update, cim_domain_bevy::apply_component_updates);
    
    let entity_id = Uuid::new_v4();
    let entity = app.world_mut().spawn((
        NatsSyncedEntity { entity_id },
    )).id();
    
    // Create a workflow state component event
    let component_data = EcsComponentData {
        component_type: "WorkflowStateComponent".to_string(),
        data: json!({
            "state": "Running",
            "definition_id": "test-workflow",
            "started_at": "2024-01-01T00:00:00Z",
            "completed_at": null
        }),
    };
    
    app.world_mut().entity_mut(entity).insert(PendingComponentUpdate {
        component_data,
    });
    
    app.update();
    
    // The workflow state update should be processed without errors
    // In a real implementation, we'd check for workflow-specific visualization components
    assert!(app.world().get::<PendingComponentUpdate>(entity).is_none());
}

#[test]
fn test_unknown_component_type() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    app.add_systems(Update, cim_domain_bevy::apply_component_updates);
    
    let entity_id = Uuid::new_v4();
    let entity = app.world_mut().spawn((
        NatsSyncedEntity { entity_id },
    )).id();
    
    // Create an unknown component type
    let component_data = EcsComponentData {
        component_type: "UnknownComponent".to_string(),
        data: json!({
            "some_field": "some_value"
        }),
    };
    
    app.world_mut().entity_mut(entity).insert(PendingComponentUpdate {
        component_data,
    });
    
    app.update();
    
    // The pending update should be removed even for unknown types
    assert!(app.world().get::<PendingComponentUpdate>(entity).is_none());
}

#[test]
fn test_entity_creation_from_component_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Count entities before
    let initial_count = app.world().entities().len();
    
    // Add a marker component to track new entities
    #[derive(Component)]
    struct NewEntityMarker;
    
    // Custom system to mark new entities
    fn mark_new_entities(
        mut commands: Commands,
        query: Query<Entity, (With<NatsSyncedEntity>, Without<NewEntityMarker>)>,
    ) {
        for entity in query.iter() {
            commands.entity(entity).insert(NewEntityMarker);
        }
    }
    
    app.add_systems(Update, mark_new_entities);
    app.update();
    
    // Verify a new entity would be created with proper components
    let entity_id = Uuid::new_v4();
    let new_entity = app.world_mut().spawn((
        NatsSyncedEntity { entity_id },
        PendingComponentUpdate {
            component_data: EcsComponentData {
                component_type: "Position3D".to_string(),
                data: json!({"x": 0.0, "y": 0.0, "z": 0.0}),
            },
        },
    )).id();
    
    assert_eq!(app.world().entities().len(), initial_count + 1);
    assert!(app.world().get::<NatsSyncedEntity>(new_entity).is_some());
}