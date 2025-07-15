//! NATS Event Visualization System
//!
//! This module provides real-time visualization of domain events flowing through NATS.
//! It subscribes to all domain event streams and creates visual representations of:
//! - Event flow between domains
//! - Causation chains
//! - Event correlation
//! - Real-time event monitoring

use bevy::prelude::*;
use bevy::render::mesh::{Mesh, Meshable};
use async_nats::Client;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

/// Plugin for NATS event visualization
pub struct NatsEventVisualizationPlugin {
    /// NATS client for subscribing to events
    pub nats_client: Arc<Client>,
    /// Maximum number of events to visualize at once
    pub max_events: usize,
    /// Event retention duration (seconds)
    pub retention_seconds: u64,
}

impl Default for NatsEventVisualizationPlugin {
    fn default() -> Self {
        Self {
            nats_client: Arc::new(Client::new()), // This would need to be properly initialized
            max_events: 100,
            retention_seconds: 300, // 5 minutes
        }
    }
}

impl Plugin for NatsEventVisualizationPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(EventVisualizationConfig {
            max_events: self.max_events,
            retention_seconds: self.retention_seconds,
        })
        .insert_resource(EventStore::new(self.max_events))
        .insert_resource(EventFlowGraph::new())
        .insert_resource(DomainColors::default());

        // Events
        app.add_event::<DomainEventReceived>()
           .add_event::<EventVisualizationCommand>();

        // Systems
        app.add_systems(Startup, setup_event_visualization)
           .add_systems(Update, (
               process_incoming_events,
               update_event_positions,
               create_event_visuals,
               update_event_connections,
               handle_event_interactions,
               cleanup_old_events,
           ).chain());

        // Start NATS subscription
        let nats_client = self.nats_client.clone();
        let (tx, rx) = mpsc::channel(1000);
        
        app.insert_resource(EventReceiver(Arc::new(RwLock::new(rx))));
        
        // Spawn async task to subscribe to NATS events
        let runtime = tokio::runtime::Handle::current();
        runtime.spawn(subscribe_to_domain_events(nats_client, tx));
    }
}

/// Configuration for event visualization
#[derive(Resource)]
struct EventVisualizationConfig {
    max_events: usize,
    retention_seconds: u64,
}

/// Domain event that was received from NATS
#[derive(Event, Debug, Clone)]
pub struct DomainEventReceived {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub payload: serde_json::Value,
}

/// Commands for controlling event visualization
#[derive(Event, Debug)]
pub enum EventVisualizationCommand {
    /// Focus on a specific event
    FocusEvent(String),
    /// Show events for a specific correlation
    ShowCorrelation(String),
    /// Filter events by domain
    FilterByDomain(String),
    /// Clear all filters
    ClearFilters,
    /// Pause/resume event processing
    TogglePause,
}

/// Store for received events
#[derive(Resource)]
pub struct EventStore {
    events: Arc<RwLock<VecDeque<DomainEventReceived>>>,
    max_events: usize,
}

impl EventStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(max_events))),
            max_events,
        }
    }

    fn add_event(&self, event: DomainEventReceived) {
        let mut events = self.events.write();
        if events.len() >= self.max_events {
            events.pop_front();
        }
        events.push_back(event);
    }

    pub fn get_all_events(&self) -> Vec<DomainEventReceived> {
        self.events.read().iter().cloned().collect()
    }

    pub fn get_recent_events(&self, seconds: u64) -> Vec<DomainEventReceived> {
        let cutoff = Utc::now() - chrono::Duration::seconds(seconds as i64);
        self.events.read()
            .iter()
            .filter(|e| e.timestamp > cutoff)
            .cloned()
            .collect()
    }
}

/// Graph structure for event relationships
#[derive(Resource, Default)]
struct EventFlowGraph {
    /// Adjacency list of event relationships
    edges: HashMap<String, Vec<String>>,
    /// Node positions for force-directed layout
    positions: HashMap<String, Vec3>,
}

impl EventFlowGraph {
    fn new() -> Self {
        Self::default()
    }

    fn add_edge(&mut self, from: String, to: String) {
        self.edges.entry(from).or_default().push(to);
    }

    fn get_connected(&self, event_id: &str) -> Vec<String> {
        self.edges.get(event_id).cloned().unwrap_or_default()
    }
}

/// Domain colors for visual differentiation
#[derive(Resource)]
struct DomainColors {
    colors: HashMap<String, Color>,
}

impl Default for DomainColors {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("graph".to_string(), Color::srgb(0.2, 0.7, 0.9));
        colors.insert("agent".to_string(), Color::srgb(0.9, 0.2, 0.2));
        colors.insert("workflow".to_string(), Color::srgb(0.2, 0.9, 0.2));
        colors.insert("document".to_string(), Color::srgb(0.9, 0.9, 0.2));
        colors.insert("person".to_string(), Color::srgb(0.9, 0.2, 0.9));
        colors.insert("organization".to_string(), Color::srgb(0.2, 0.9, 0.9));
        colors.insert("location".to_string(), Color::srgb(0.5, 0.5, 0.9));
        colors.insert("dialog".to_string(), Color::srgb(0.9, 0.5, 0.2));
        colors.insert("policy".to_string(), Color::srgb(0.5, 0.9, 0.5));
        colors.insert("git".to_string(), Color::srgb(0.7, 0.4, 0.1));
        colors.insert("nix".to_string(), Color::srgb(0.4, 0.6, 0.8));
        colors.insert("conceptual_spaces".to_string(), Color::srgb(0.8, 0.3, 0.8));
        colors.insert("identity".to_string(), Color::srgb(0.6, 0.8, 0.4));
        Self { colors }
    }
}

/// Component for event visual entities
#[derive(Component)]
struct EventVisual {
    event_id: String,
    domain: String,
    event_type: String,
    timestamp: DateTime<Utc>,
    correlation_id: Option<String>,
}

/// Component for event connection lines
#[derive(Component)]
struct EventConnection {
    from_event: String,
    to_event: String,
    connection_type: ConnectionType,
}

#[derive(Debug, Clone)]
enum ConnectionType {
    Causation,
    Correlation,
    Temporal,
}

/// Receiver for events from NATS
#[derive(Resource)]
struct EventReceiver(Arc<RwLock<mpsc::Receiver<DomainEventReceived>>>);

/// Setup event visualization
fn setup_event_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ground plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -5.0, 0.0),
    ));

    // Add UI instructions
    commands.spawn((
        Text::new(
            "NATS Event Visualization\n\
            Click events to focus\n\
            Mouse wheel to zoom\n\
            Right click + drag to rotate"
        ),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));
}

/// Process incoming events from NATS
fn process_incoming_events(
    event_receiver: Res<EventReceiver>,
    event_store: Res<EventStore>,
    mut event_writer: EventWriter<DomainEventReceived>,
    mut event_graph: ResMut<EventFlowGraph>,
) {
    let mut receiver = event_receiver.0.write();
    
    // Process up to 10 events per frame to avoid blocking
    for _ in 0..10 {
        match receiver.try_recv() {
            Ok(event) => {
                // Update event graph
                if let Some(causation_id) = &event.causation_id {
                    event_graph.add_edge(causation_id.clone(), event.event_id.clone());
                }
                
                // Store and emit event
                event_store.add_event(event.clone());
                event_writer.write(event);
            }
            Err(mpsc::error::TryRecvError::Empty) => break,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                warn!("NATS event receiver disconnected");
                break;
            }
        }
    }
}

/// Create visual representations for new events
fn create_event_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event_reader: EventReader<DomainEventReceived>,
    domain_colors: Res<DomainColors>,
    mut event_graph: ResMut<EventFlowGraph>,
) {
    for event in event_reader.read() {
        let color = domain_colors.colors
            .get(&event.domain)
            .copied()
            .unwrap_or(Color::srgb(0.5, 0.5, 0.5));

        // Calculate initial position (will be updated by force-directed layout)
        let initial_pos = Vec3::new(
            (rand::random::<f32>() - 0.5) * 20.0,
            (rand::random::<f32>() - 0.5) * 10.0 + 5.0,
            (rand::random::<f32>() - 0.5) * 20.0,
        );

        event_graph.positions.insert(event.event_id.clone(), initial_pos);

        // Spawn event sphere
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color.into(),
                ..default()
            })),
            Transform::from_translation(initial_pos),
            EventVisual {
                event_id: event.event_id.clone(),
                domain: event.domain.clone(),
                event_type: event.event_type.clone(),
                timestamp: event.timestamp,
                correlation_id: event.correlation_id.clone(),
            },
        ));

        // Spawn event label
        commands.spawn((
            Text::new(format!("{}\n{}", event.domain, event.event_type)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Transform::from_translation(initial_pos + Vec3::Y * 1.0),
        ));
    }
}

/// Update event positions using force-directed layout
fn update_event_positions(
    mut event_graph: ResMut<EventFlowGraph>,
    mut query: Query<(&EventVisual, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut forces: HashMap<String, Vec3> = HashMap::new();

    // Calculate repulsive forces between all events
    let positions: Vec<(String, Vec3)> = query.iter()
        .map(|(ev, t)| (ev.event_id.clone(), t.translation))
        .collect();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let (id1, pos1) = &positions[i];
            let (id2, pos2) = &positions[j];
            
            let delta = *pos2 - *pos1;
            let distance = delta.length().max(0.1);
            let force_magnitude = 10.0 / (distance * distance);
            let force = delta.normalize() * force_magnitude;

            *forces.entry(id1.clone()).or_default() -= force;
            *forces.entry(id2.clone()).or_default() += force;
        }
    }

    // Calculate attractive forces for connected events
    for (from_id, to_ids) in &event_graph.edges {
        if let Some(from_pos) = event_graph.positions.get(from_id) {
            for to_id in to_ids {
                if let Some(to_pos) = event_graph.positions.get(to_id) {
                    let delta = *to_pos - *from_pos;
                    let distance = delta.length().max(0.1);
                    let force_magnitude = distance * 0.1;
                    let force = delta.normalize() * force_magnitude;

                    *forces.entry(from_id.clone()).or_default() += force;
                    *forces.entry(to_id.clone()).or_default() -= force;
                }
            }
        }
    }

    // Apply forces
    for (event, mut transform) in query.iter_mut() {
        if let Some(force) = forces.get(&event.event_id) {
            let velocity = *force * dt;
            transform.translation += velocity;
            
            // Update stored position
            event_graph.positions.insert(event.event_id.clone(), transform.translation);
        }
    }
}

/// Update visual connections between events
fn update_event_connections(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    event_graph: Res<EventFlowGraph>,
    event_positions: Query<(&EventVisual, &Transform)>,
    connections: Query<Entity, With<EventConnection>>,
) {
    // Remove old connections
    for entity in connections.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create new connections
    let pos_map: HashMap<String, Vec3> = event_positions.iter()
        .map(|(ev, t)| (ev.event_id.clone(), t.translation))
        .collect();

    for (from_id, to_ids) in &event_graph.edges {
        if let Some(from_pos) = pos_map.get(from_id) {
            for to_id in to_ids {
                if let Some(to_pos) = pos_map.get(to_id) {
                    let midpoint = (*from_pos + *to_pos) / 2.0;
                    let direction = *to_pos - *from_pos;
                    let distance = direction.length();
                    
                    if distance > 0.01 {
                        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
                        
                        commands.spawn((
                            Mesh3d(meshes.add(Cylinder::new(0.05, distance).mesh())),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgba(0.8, 0.8, 0.8, 0.5),
                                alpha_mode: AlphaMode::Blend,
                                ..default()
                            })),
                            Transform::from_translation(midpoint)
                                .with_rotation(rotation),
                            EventConnection {
                                from_event: from_id.clone(),
                                to_event: to_id.clone(),
                                connection_type: ConnectionType::Causation,
                            },
                        ));
                    }
                }
            }
        }
    }
}

/// Handle mouse interactions with events
fn handle_event_interactions(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    events: Query<(&EventVisual, &Transform)>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                for (camera, camera_transform) in cameras.iter() {
                    if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        for (event, transform) in events.iter() {
                            if ray_intersects_sphere(ray.origin, ray.direction.as_vec3(), transform.translation, 0.5) {
                                info!("Clicked event: {} - {}", event.domain, event.event_type);
                                // TODO: Implement focus/details view
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Clean up old events based on retention policy
fn cleanup_old_events(
    mut commands: Commands,
    config: Res<EventVisualizationConfig>,
    events: Query<(Entity, &EventVisual)>,
    connections: Query<(Entity, &EventConnection)>,
) {
    let cutoff = Utc::now() - chrono::Duration::seconds(config.retention_seconds as i64);
    
    let mut removed_events = Vec::new();
    
    // Remove old event visuals
    for (entity, event) in events.iter() {
        if event.timestamp < cutoff {
            commands.entity(entity).despawn();
            removed_events.push(event.event_id.clone());
        }
    }
    
    // Remove connections involving removed events
    for (entity, connection) in connections.iter() {
        if removed_events.contains(&connection.from_event) || 
           removed_events.contains(&connection.to_event) {
            commands.entity(entity).despawn();
        }
    }
}

/// Subscribe to domain events from NATS
async fn subscribe_to_domain_events(
    client: Arc<Client>,
    tx: mpsc::Sender<DomainEventReceived>,
) {
    // Subscribe to all domain events
    let subject = "*.*.event.v1"; // Pattern: domain.aggregate.event.version
    
    match client.subscribe(subject).await {
        Ok(mut subscriber) => {
            info!("Subscribed to NATS events on: {}", subject);
            
            while let Some(msg) = subscriber.next().await {
                // Parse subject to extract domain and event type
                let parts: Vec<&str> = msg.subject.split('.').collect();
                if parts.len() >= 3 {
                    let domain = parts[0].to_string();
                    let aggregate_type = parts[1].to_string();
                    let event_type = parts[2].to_string();
                    
                    // Try to parse the event payload
                    if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                        let event = DomainEventReceived {
                            event_id: payload.get("event_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&Uuid::new_v4().to_string())
                                .to_string(),
                            timestamp: payload.get("timestamp")
                                .and_then(|v| v.as_str())
                                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(Utc::now),
                            domain,
                            event_type,
                            aggregate_id: payload.get("aggregate_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            aggregate_type,
                            correlation_id: payload.get("correlation_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            causation_id: payload.get("causation_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            payload,
                        };
                        
                        if let Err(e) = tx.send(event).await {
                            error!("Failed to send event to visualization: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to subscribe to NATS events: {}", e);
        }
    }
}

/// Helper function to check ray-sphere intersection
fn ray_intersects_sphere(ray_origin: Vec3, ray_dir: Vec3, sphere_center: Vec3, sphere_radius: f32) -> bool {
    let oc = ray_origin - sphere_center;
    let a = ray_dir.dot(ray_dir);
    let b = 2.0 * oc.dot(ray_dir);
    let c = oc.dot(oc) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}

/// Helper to generate random float
mod rand {
    pub fn random<T>() -> T 
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        use rand::Rng;
        rand::thread_rng().gen()
    }
}