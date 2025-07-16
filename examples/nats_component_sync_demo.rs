//! Example demonstrating NATS-based component synchronization
//!
//! This example shows how DDD components in domain modules are automatically
//! synchronized with Bevy ECS components via NATS messaging.

use bevy::prelude::*;
use cim_domain_bevy::{
    AsyncSyncBridge, GraphVisualizationPlugin, NatsComponentPlugin, NatsSyncedEntity,
};
use cim_domain::{ComponentEvent, EcsComponentData, DomainComponentSync};
use async_nats::Client;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn main() {
    // Create tokio runtime for async operations
    let runtime = Runtime::new().expect("Failed to create runtime");
    
    // Connect to NATS
    let nats_client = runtime.block_on(async {
        Client::connect("nats://localhost:4222")
            .await
            .expect("Failed to connect to NATS")
    });
    let nats_client = Arc::new(nats_client);

    // Create the async-sync bridge
    let bridge = AsyncSyncBridge::new(1000);

    // Create Bevy app
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphVisualizationPlugin::default())
        .add_plugins(NatsComponentPlugin::new(nats_client.clone()))
        .insert_resource(bridge.clone())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            simulate_domain_updates,
            log_synced_entities,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Create a few entities that will be synced via NATS
    for i in 0..3 {
        let entity_id = Uuid::new_v4();
        
        commands.spawn((
            Pbr {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 1.0),
                    ..default()
                }),
                ..default()
            },
            Transform::from_xyz(i as f32 * 2.0 - 2.0, 0.0, 0.0),
            NatsSyncedEntity { entity_id },
            Name::new(format!("Synced Entity {}", i)),
        ));
    }
}

/// System that simulates domain component updates
fn simulate_domain_updates(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Press SPACE to trigger a simulated component update
    if keyboard.just_pressed(KeyCode::Space) {
        info!("Simulating domain component update...");
        
        // In a real application, this would come from a domain module
        // For demo purposes, we'll publish directly to NATS
        
        // TODO: Publish ComponentEvent to NATS
        // This would normally be done by DomainComponentSync in the domain layer
    }
}

/// System that logs information about synced entities
fn log_synced_entities(
    query: Query<(Entity, &NatsSyncedEntity, &Transform), Changed<Transform>>,
) {
    for (entity, synced, transform) in query.iter() {
        info!(
            "Entity {:?} (UUID: {}) position updated to: {:?}",
            entity, synced.entity_id, transform.translation
        );
    }
}