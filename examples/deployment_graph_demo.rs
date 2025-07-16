//! Deployment Graph Visualization Demo
//!
//! This demo shows how to create and visualize deployment graphs in Bevy:
//! - Create deployment nodes (services, databases, agents)
//! - Show dependencies and connections
//! - Display deployment metadata
//! - Interactive graph manipulation
//!
//! Run with: cargo run --example deployment_graph_demo --package cim-domain-bevy

use bevy::prelude::*;
use cim_domain_bevy::*;
use cim_domain_graph::{
    aggregate::business_graph::Graph,
    deployment::{
        DeploymentNodeType, DeploymentEdgeType, ResourceRequirements,
        DatabaseEngine, MessageBusType, LoadBalancingStrategy,
        graph_adapter::create_deployment_node_metadata,
    },
    GraphId, NodeId, EdgeId,
};
use std::collections::HashMap;

/// Demo state for deployment visualization
#[derive(Resource)]
struct DeploymentDemoState {
    graph: Graph,
    selected_node: Option<NodeId>,
    show_metadata: bool,
}

impl Default for DeploymentDemoState {
    fn default() -> Self {
        Self {
            graph: Graph::new(
                GraphId::new(),
                "CIM Leaf Deployment".to_string(),
                "Example deployment configuration".to_string(),
            ),
            selected_node: None,
            show_metadata: true,
        }
    }
}

/// Map node IDs to Bevy entities
#[derive(Resource, Default)]
struct NodeEntityMap(HashMap<NodeId, Entity>);

/// Component for deployment node visuals
#[derive(Component)]
struct DeploymentNodeVisual {
    node_id: NodeId,
    node_type: String,
}

/// Component for UI text
#[derive(Component)]
struct MetadataText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM Deployment Graph Visualization".to_string(),
                resolution: (1400.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CimVizPlugin::default())
        .insert_resource(DeploymentDemoState::default())
        .insert_resource(NodeEntityMap::default())
        .add_systems(Startup, (setup_scene, create_deployment_graph))
        .add_systems(Update, (
            visualize_deployment_nodes,
            visualize_deployment_edges,
            handle_node_selection,
            update_metadata_display,
            handle_keyboard_input,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        GraphCamera,
    ));

    // Lights
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 1500000.0,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(Circle::new(50.0).mesh().build()),
        MeshMaterial3d(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..default()
        }),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // UI setup
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        MetadataText,
    ));

    // Instructions
    commands.spawn((
        Text::new(
            "Deployment Graph Visualization\n\
            - Click nodes to select\n\
            - Press 'M' to toggle metadata\n\
            - Press 'L' to change layout\n\
            - Press 'Space' to add random node"
        ),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn create_deployment_graph(
    mut state: ResMut<DeploymentDemoState>,
) {
    let graph = &mut state.graph;
    
    // Create deployment nodes
    
    // 1. Load Balancer
    let lb_id = NodeId::new();
    let lb_node = DeploymentNodeType::LoadBalancer {
        name: "nginx-lb".to_string(),
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: std::time::Duration::from_secs(5),
        backends: vec!["web-api-1".to_string(), "web-api-2".to_string()],
    };
    graph.add_node(
        lb_id,
        "LoadBalancer".to_string(),
        create_deployment_node_metadata(lb_node),
    ).unwrap();
    
    // 2. Web API Services
    let api1_id = NodeId::new();
    let api1_node = DeploymentNodeType::Service {
        name: "web-api-1".to_string(),
        command: "cargo run --bin api".to_string(),
        args: vec!["--port".to_string(), "8080".to_string()],
        environment: {
            let mut env = HashMap::new();
            env.insert("DATABASE_URL".to_string(), "postgresql://db:5432/myapp".to_string());
            env.insert("NATS_URL".to_string(), "nats://nats:4222".to_string());
            env
        },
        port: Some(8080),
        health_check: None,
        resources: ResourceRequirements {
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            disk_gb: Some(10),
        },
    };
    graph.add_node(
        api1_id,
        "Service".to_string(),
        create_deployment_node_metadata(api1_node),
    ).unwrap();
    
    let api2_id = NodeId::new();
    let api2_node = DeploymentNodeType::Service {
        name: "web-api-2".to_string(),
        command: "cargo run --bin api".to_string(),
        args: vec!["--port".to_string(), "8081".to_string()],
        environment: {
            let mut env = HashMap::new();
            env.insert("DATABASE_URL".to_string(), "postgresql://db:5432/myapp".to_string());
            env.insert("NATS_URL".to_string(), "nats://nats:4222".to_string());
            env
        },
        port: Some(8081),
        health_check: None,
        resources: ResourceRequirements {
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            disk_gb: Some(10),
        },
    };
    graph.add_node(
        api2_id,
        "Service".to_string(),
        create_deployment_node_metadata(api2_node),
    ).unwrap();
    
    // 3. Database
    let db_id = NodeId::new();
    let db_node = DeploymentNodeType::Database {
        name: "postgres-db".to_string(),
        engine: DatabaseEngine::PostgreSQL,
        version: "15".to_string(),
        persistent: true,
        backup_schedule: Some("0 2 * * *".to_string()),
        resources: ResourceRequirements {
            cpu_cores: Some(2.0),
            memory_mb: Some(2048),
            disk_gb: Some(100),
        },
    };
    graph.add_node(
        db_id,
        "Database".to_string(),
        create_deployment_node_metadata(db_node),
    ).unwrap();
    
    // 4. Message Bus (NATS)
    let nats_id = NodeId::new();
    let nats_node = DeploymentNodeType::MessageBus {
        name: "nats-cluster".to_string(),
        bus_type: MessageBusType::NATS,
        cluster_size: 3,
        persistence: true,
        topics: vec![],
    };
    graph.add_node(
        nats_id,
        "MessageBus".to_string(),
        create_deployment_node_metadata(nats_node),
    ).unwrap();
    
    // 5. AI Agent
    let agent_id = NodeId::new();
    let agent_node = DeploymentNodeType::Agent {
        name: "deployment-agent".to_string(),
        capabilities: vec!["deploy".to_string(), "monitor".to_string(), "rollback".to_string()],
        subscriptions: vec!["deployment.*".to_string(), "monitoring.*".to_string()],
        rate_limit: None,
        resources: ResourceRequirements {
            cpu_cores: Some(0.5),
            memory_mb: Some(256),
            disk_gb: None,
        },
    };
    graph.add_node(
        agent_id,
        "Agent".to_string(),
        create_deployment_node_metadata(agent_node),
    ).unwrap();
    
    // Create edges (dependencies and connections)
    
    // Load balancer -> APIs
    graph.add_edge(
        EdgeId::new(),
        lb_id,
        api1_id,
        "LoadBalances".to_string(),
        HashMap::new(),
    ).unwrap();
    
    graph.add_edge(
        EdgeId::new(),
        lb_id,
        api2_id,
        "LoadBalances".to_string(),
        HashMap::new(),
    ).unwrap();
    
    // APIs -> Database
    graph.add_edge(
        EdgeId::new(),
        api1_id,
        db_id,
        "DependsOn".to_string(),
        HashMap::new(),
    ).unwrap();
    
    graph.add_edge(
        EdgeId::new(),
        api2_id,
        db_id,
        "DependsOn".to_string(),
        HashMap::new(),
    ).unwrap();
    
    // APIs -> NATS
    graph.add_edge(
        EdgeId::new(),
        api1_id,
        nats_id,
        "ConnectsTo".to_string(),
        HashMap::new(),
    ).unwrap();
    
    graph.add_edge(
        EdgeId::new(),
        api2_id,
        nats_id,
        "ConnectsTo".to_string(),
        HashMap::new(),
    ).unwrap();
    
    // Agent -> NATS
    graph.add_edge(
        EdgeId::new(),
        agent_id,
        nats_id,
        "SubscribesTo".to_string(),
        HashMap::new(),
    ).unwrap();
    
    // Agent manages services
    graph.add_edge(
        EdgeId::new(),
        agent_id,
        api1_id,
        "Manages".to_string(),
        HashMap::new(),
    ).unwrap();
    
    graph.add_edge(
        EdgeId::new(),
        agent_id,
        api2_id,
        "Manages".to_string(),
        HashMap::new(),
    ).unwrap();
}

fn visualize_deployment_nodes(
    state: Res<DeploymentDemoState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeEntityMap>,
    existing_nodes: Query<Entity, With<DeploymentNodeVisual>>,
) {
    // Remove existing nodes
    for entity in existing_nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
    node_map.0.clear();
    
    // Create node visuals
    let nodes = state.graph.nodes();
    let node_count = nodes.len();
    
    for (i, (node_id, node)) in nodes.iter().enumerate() {
        // Calculate position in a circle
        let angle = (i as f32 / node_count as f32) * std::f32::consts::TAU;
        let radius = 10.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        
        // Get node color based on type
        let (color, shape) = match node.node_type.as_str() {
            "LoadBalancer" => (Color::srgb(0.2, 0.7, 0.9), NodeShape::Cube),
            "Service" => (Color::srgb(0.2, 0.9, 0.2), NodeShape::Sphere),
            "Database" => (Color::srgb(0.9, 0.9, 0.2), NodeShape::Cylinder),
            "MessageBus" => (Color::srgb(0.9, 0.2, 0.9), NodeShape::Torus),
            "Agent" => (Color::srgb(0.9, 0.5, 0.2), NodeShape::Cone),
            _ => (Color::srgb(0.5, 0.5, 0.5), NodeShape::Sphere),
        };
        
        // Create mesh based on shape
        let mesh = match shape {
            NodeShape::Sphere => meshes.add(Sphere::new(1.0).mesh()),
            NodeShape::Cube => meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh()),
            NodeShape::Cylinder => meshes.add(Cylinder::new(1.0, 2.0).mesh()),
            NodeShape::Torus => meshes.add(Torus::new(0.8, 0.3).mesh()),
            NodeShape::Cone => meshes.add(Cone::new(1.0, 2.0).mesh()),
        };
        
        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.3,
                perceptual_roughness: 0.5,
                ..default()
            })),
            Transform::from_xyz(x, 0.0, z),
            DeploymentNodeVisual {
                node_id: *node_id,
                node_type: node.node_type.clone(),
            },
            NodeVisual {
                node_id: *node_id,
                graph_id: state.graph.id(),
            },
        )).id();
        
        // Add label
        if let Some(deployment_data) = node.metadata.get("deployment") {
            if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(deployment_data.clone()) {
                let name = node_type.name();
                commands.spawn((
                    Text::new(name),
                    Transform::from_xyz(x, 2.5, z),
                ));
            }
        }
        
        node_map.0.insert(*node_id, entity);
    }
}

fn visualize_deployment_edges(
    state: Res<DeploymentDemoState>,
    node_map: Res<NodeEntityMap>,
    mut gizmos: Gizmos,
    transforms: Query<&Transform>,
) {
    for edge in state.graph.edges().values() {
        if let (Some(&from_entity), Some(&to_entity)) = (
            node_map.0.get(&edge.source_id),
            node_map.0.get(&edge.target_id),
        ) {
            if let (Ok(from_transform), Ok(to_transform)) = (
                transforms.get(from_entity),
                transforms.get(to_entity),
            ) {
                let color = match edge.edge_type.as_str() {
                    "DependsOn" => Color::srgb(0.9, 0.2, 0.2),
                    "ConnectsTo" => Color::srgb(0.2, 0.9, 0.2),
                    "LoadBalances" => Color::srgb(0.2, 0.2, 0.9),
                    "Manages" => Color::srgb(0.9, 0.9, 0.2),
                    "SubscribesTo" => Color::srgb(0.9, 0.2, 0.9),
                    _ => Color::srgb(0.5, 0.5, 0.5),
                };
                
                gizmos.line(
                    from_transform.translation,
                    to_transform.translation,
                    color,
                );
                
                // Draw arrow head
                let direction = (to_transform.translation - from_transform.translation).normalize();
                let arrow_pos = to_transform.translation - direction * 1.5;
                gizmos.sphere(arrow_pos, Quat::IDENTITY, 0.2, color);
            }
        }
    }
}

fn handle_node_selection(
    mut state: ResMut<DeploymentDemoState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(&Transform, &DeploymentNodeVisual)>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_position) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = cameras.get_single() else { return };
    
    if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
        let mut closest_distance = f32::MAX;
        let mut selected = None;
        
        for (transform, node_visual) in nodes.iter() {
            let distance = ray.closest_point_to_point(transform.translation);
            let dist_to_node = distance.distance(transform.translation);
            
            if dist_to_node < 2.0 && dist_to_node < closest_distance {
                closest_distance = dist_to_node;
                selected = Some(node_visual.node_id);
            }
        }
        
        state.selected_node = selected;
    }
}

fn update_metadata_display(
    state: Res<DeploymentDemoState>,
    mut text_query: Query<&mut Text, With<MetadataText>>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };
    
    if !state.show_metadata {
        text.0 = "Metadata display: OFF (press M to toggle)".to_string();
        return;
    }
    
    let mut display_text = "=== Deployment Graph ===\n\n".to_string();
    
    if let Some(selected_id) = state.selected_node {
        if let Some(node) = state.graph.nodes().get(&selected_id) {
            display_text.push_str(&format!("Selected: {} ({})\n", selected_id, node.node_type));
            
            if let Some(deployment_data) = node.metadata.get("deployment") {
                if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(deployment_data.clone()) {
                    display_text.push_str(&format!("\nDeployment Details:\n"));
                    
                    match node_type {
                        DeploymentNodeType::Service { name, port, resources, environment, .. } => {
                            display_text.push_str(&format!("  Name: {}\n", name));
                            if let Some(p) = port {
                                display_text.push_str(&format!("  Port: {}\n", p));
                            }
                            display_text.push_str(&format!("  CPU: {:?} cores\n", resources.cpu_cores));
                            display_text.push_str(&format!("  Memory: {:?} MB\n", resources.memory_mb));
                            if !environment.is_empty() {
                                display_text.push_str("  Environment:\n");
                                for (k, v) in environment {
                                    display_text.push_str(&format!("    {}: {}\n", k, v));
                                }
                            }
                        }
                        DeploymentNodeType::Database { name, engine, version, persistent, .. } => {
                            display_text.push_str(&format!("  Name: {}\n", name));
                            display_text.push_str(&format!("  Engine: {:?}\n", engine));
                            display_text.push_str(&format!("  Version: {}\n", version));
                            display_text.push_str(&format!("  Persistent: {}\n", persistent));
                        }
                        DeploymentNodeType::Agent { name, capabilities, .. } => {
                            display_text.push_str(&format!("  Name: {}\n", name));
                            display_text.push_str(&format!("  Capabilities: {}\n", capabilities.join(", ")));
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        display_text.push_str("Click a node to see deployment details");
    }
    
    text.0 = display_text;
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DeploymentDemoState>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        state.show_metadata = !state.show_metadata;
    }
}

#[derive(Clone, Copy)]
enum NodeShape {
    Sphere,
    Cube,
    Cylinder,
    Torus,
    Cone,
}