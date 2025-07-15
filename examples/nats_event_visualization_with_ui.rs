//! Enhanced NATS Event Visualization Demo with Filtering UI and Statistics
//!
//! This example demonstrates the NATS event visualization system with:
//! - Real-time 3D visualization of domain events
//! - Filtering UI for domain, event type, and search
//! - Live statistics display
//! - Event correlation visualization

use bevy::prelude::*;
use cim_domain_bevy::{
    NatsEventVisualizationPlugin, 
    EventVisualizationUIPlugin,
    DomainEventReceived,
};
use async_nats::Client;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn main() {
    // Create async runtime for NATS
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    
    // Connect to NATS (use default URL or from environment)
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    let nats_client = runtime.block_on(async {
        match Client::connect(&nats_url).await {
            Ok(client) => {
                println!("✅ Connected to NATS at {}", nats_url);
                println!("   Subscribing to domain events...");
                Arc::new(client)
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to NATS: {}", e);
                eprintln!("   Demo will run with simulated events");
                // Create a dummy client for demo purposes
                Arc::new(Client::new())
            }
        }
    });

    // Setup Bevy app with visualization plugins
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NATS Event Visualization with UI".to_string(),
                resolution: (1600.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NatsEventVisualizationPlugin {
            nats_client: nats_client.clone(),
            max_events: 200,
            retention_seconds: 600, // 10 minutes
        })
        .add_plugins(EventVisualizationUIPlugin)
        .add_systems(Startup, setup_demo_instructions)
        .add_systems(Update, (
            handle_demo_controls,
            generate_demo_events.run_if(resource_exists::<DemoMode>),
        ))
        .run();
}

/// Demo mode resource for generating test events
#[derive(Resource)]
struct DemoMode {
    enabled: bool,
    event_rate: f32, // Events per second
    last_event_time: f32,
}

impl Default for DemoMode {
    fn default() -> Self {
        Self {
            enabled: false,
            event_rate: 2.0,
            last_event_time: 0.0,
        }
    }
}

/// Setup demo instructions
fn setup_demo_instructions(mut commands: Commands) {
    // Add help text
    commands.spawn((
        Text::new(
            "Enhanced NATS Event Visualization\n\
            \n\
            Controls:\n\
            • Left Panel: Filter events by domain, type, or search\n\
            • Right Panel: Live statistics\n\
            • Click events to see details\n\
            • Mouse wheel: Zoom\n\
            • Right click + drag: Rotate view\n\
            • D: Toggle demo mode (simulated events)\n\
            • R: Reset view\n\
            • ESC: Exit"
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.9, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(320.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));

    // Initialize demo mode (disabled by default)
    commands.insert_resource(DemoMode::default());
}

/// Handle demo controls
fn handle_demo_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut demo_mode: ResMut<DemoMode>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // Toggle demo mode
    if keyboard.just_pressed(KeyCode::KeyD) {
        demo_mode.enabled = !demo_mode.enabled;
        println!("Demo mode: {}", if demo_mode.enabled { "ON" } else { "OFF" });
    }

    // Reset camera view
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok(mut transform) = camera_query.get_single_mut() {
            *transform = Transform::from_xyz(0.0, 20.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y);
        }
    }

    // Exit on ESC
    if keyboard.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

/// Generate demo events when in demo mode
fn generate_demo_events(
    time: Res<Time>,
    mut demo_mode: ResMut<DemoMode>,
    mut event_writer: EventWriter<DomainEventReceived>,
) {
    if !demo_mode.enabled {
        return;
    }

    let current_time = time.elapsed_secs();
    let time_since_last = current_time - demo_mode.last_event_time;

    // Generate events based on rate
    if time_since_last >= 1.0 / demo_mode.event_rate {
        demo_mode.last_event_time = current_time;

        // Create a variety of demo events
        let domains = vec!["workflow", "agent", "document", "git", "policy"];
        let event_types = vec!["created", "updated", "executed", "completed", "failed"];
        
        let domain = domains[rand::random::<usize>() % domains.len()];
        let event_type = event_types[rand::random::<usize>() % event_types.len()];
        
        let event_id = format!("demo-{}", uuid::Uuid::new_v4());
        let aggregate_id = format!("agg-{}", rand::random::<u32>() % 100);
        
        // Create causation chains sometimes
        let causation_id = if rand::random::<f32>() > 0.5 {
            Some(format!("demo-{}", uuid::Uuid::new_v4()))
        } else {
            None
        };
        
        // Create correlation groups sometimes
        let correlation_id = if rand::random::<f32>() > 0.7 {
            Some(format!("corr-{}", rand::random::<u32>() % 10))
        } else {
            None
        };

        let demo_event = DomainEventReceived {
            event_id: event_id.clone(),
            timestamp: chrono::Utc::now(),
            domain: domain.to_string(),
            event_type: format!("{}_{}", domain, event_type),
            aggregate_id,
            aggregate_type: format!("{}_aggregate", domain),
            correlation_id,
            causation_id,
            payload: serde_json::json!({
                "demo": true,
                "value": rand::random::<f32>() * 100.0,
                "message": format!("Demo {} event", event_type),
            }),
        };

        event_writer.send(demo_event);
        
        // Sometimes create follow-up events to show causation chains
        if rand::random::<f32>() > 0.6 {
            let follow_up = DomainEventReceived {
                event_id: format!("demo-{}", uuid::Uuid::new_v4()),
                timestamp: chrono::Utc::now(),
                domain: "workflow".to_string(),
                event_type: "workflow_triggered".to_string(),
                aggregate_id: format!("wf-{}", rand::random::<u32>() % 50),
                aggregate_type: "workflow_aggregate".to_string(),
                correlation_id: correlation_id.clone(),
                causation_id: Some(event_id),
                payload: serde_json::json!({
                    "triggered_by": domain,
                    "action": "automated_response",
                }),
            };
            event_writer.send(follow_up);
        }
    }
}

// Required for uuid in wasm32 target
#[cfg(target_arch = "wasm32")]
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Self }
    }
    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "demo-uuid")
        }
    }
}