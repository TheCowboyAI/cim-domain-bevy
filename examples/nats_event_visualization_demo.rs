//! Demo of NATS event visualization
//!
//! This example demonstrates real-time visualization of domain events
//! flowing through NATS. It connects to a NATS server and visualizes
//! events as they occur in the system.

use bevy::prelude::*;
use cim_domain_bevy::{NatsEventVisualizationPlugin, EventVisualizationCommand};
use async_nats::Client;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Connect to NATS server
    let nats_client = match Client::connect("nats://localhost:4222").await {
        Ok(client) => {
            println!("Connected to NATS server at localhost:4222");
            Arc::new(client)
        }
        Err(e) => {
            eprintln!("Failed to connect to NATS server: {}", e);
            eprintln!("Please ensure NATS server is running on localhost:4222");
            eprintln!("On NixOS: systemctl start nats");
            eprintln!("Or add to configuration.nix: services.nats.enable = true;");
            return;
        }
    };

    // Initialize Bevy app with NATS event visualization
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NatsEventVisualizationPlugin {
            nats_client,
            max_events: 200,
            retention_seconds: 600, // Keep events for 10 minutes
        })
        .add_systems(Update, (
            camera_controls,
            keyboard_commands,
            display_stats,
        ))
        .run();
}

/// Camera controls for better visualization
fn camera_controls(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let speed = 20.0;
    let rotation_speed = 2.0;
    let dt = time.delta_secs();

    for mut transform in cameras.iter_mut() {
        // Keyboard movement
        let mut movement = Vec3::ZERO;
        
        if keyboard.pressed(KeyCode::KeyW) {
            movement.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement.x += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            movement.y += 1.0;
        }

        // Apply movement relative to camera orientation
        let forward = transform.forward();
        let right = transform.right();
        let up = transform.up();
        
        transform.translation += (forward * movement.z + right * movement.x + up * movement.y) * speed * dt;

        // Mouse rotation when right button is held
        if mouse.pressed(MouseButton::Right) {
            for motion in mouse_motion.read() {
                let yaw = -motion.delta.x * rotation_speed * dt;
                let pitch = -motion.delta.y * rotation_speed * dt;
                
                // Rotate around global Y axis for yaw
                transform.rotate_y(yaw);
                
                // Rotate around local X axis for pitch
                transform.rotate_local_x(pitch);
            }
        }
    }
}

/// Keyboard commands for controlling visualization
fn keyboard_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: EventWriter<EventVisualizationCommand>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        commands.send(EventVisualizationCommand::TogglePause);
        println!("Toggled pause");
    }
    
    if keyboard.just_pressed(KeyCode::KeyC) {
        commands.send(EventVisualizationCommand::ClearFilters);
        println!("Cleared filters");
    }
    
    // Domain filtering shortcuts
    if keyboard.just_pressed(KeyCode::Digit1) {
        commands.send(EventVisualizationCommand::FilterByDomain("graph".to_string()));
        println!("Filtering by graph domain");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        commands.send(EventVisualizationCommand::FilterByDomain("agent".to_string()));
        println!("Filtering by agent domain");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        commands.send(EventVisualizationCommand::FilterByDomain("workflow".to_string()));
        println!("Filtering by workflow domain");
    }
}

/// Display statistics about events
fn display_stats(
    mut commands: Commands,
    events: Query<Entity, With<cim_domain_bevy::nats_event_visualization::EventVisual>>,
    stats_text: Query<Entity, With<StatsText>>,
) {
    let event_count = events.iter().count();
    
    // Update or create stats text
    if let Ok(entity) = stats_text.get_single() {
        commands.entity(entity).despawn_recursive();
    }
    
    commands.spawn((
        Text::new(format!(
            "Events: {}\n\
            Controls:\n\
            WASD/QE - Move camera\n\
            Right Mouse + Drag - Rotate\n\
            P - Pause/Resume\n\
            C - Clear filters\n\
            1-3 - Filter by domain",
            event_count
        )),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        StatsText,
    ));
}

#[derive(Component)]
struct StatsText;