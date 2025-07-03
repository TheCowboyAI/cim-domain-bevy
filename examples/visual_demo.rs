//! Visual Demo of CIM-Domain-Bevy
//!
//! This demo shows a more complete visualization with:
//! - Interactive node creation/deletion
//! - Edge rendering
//! - Mouse interaction
//! - Keyboard controls
//!
//! Run with: cargo run --example visual_demo --package cim-domain-bevy

use bevy::prelude::*;
use cim_domain_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use std::collections::HashMap;

/// Animated transition component
#[derive(Component)]
struct AnimatedTransition {
    start_position: Vec3,
    target_position: Vec3,
    progress: f32,
    duration: f32,
}

/// Camera controller component
#[derive(Component)]
struct CameraController {
    pub default_position: Vec3,
    pub default_target: Vec3,
}

/// Node material handle storage
#[derive(Component)]
struct NodeMaterial(Handle<StandardMaterial>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM Visual Demo".to_string(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CimVizPlugin::default())
        .insert_resource(DemoState::default())
        .insert_resource(NodeEntityMap::default())
        .add_systems(Startup, (setup_scene, create_demo_graph))
        .add_systems(Update, (
            handle_node_creation,
            handle_edge_creation,
            handle_mouse_interaction,
            handle_keyboard_input,
            animate_nodes,
            render_edges,
            update_info_text,
        ))
        .run();
}

/// Demo state resource
#[derive(Resource, Default)]
struct DemoState {
    current_graph_id: Option<GraphId>,
    selected_node: Option<NodeId>,
    hovering_node: Option<NodeId>,
    node_count: usize,
    edge_count: usize,
}

/// Map of node IDs to entities
#[derive(Resource, Default)]
struct NodeEntityMap {
    map: HashMap<NodeId, Entity>,
}

/// Info text marker
#[derive(Component)]
struct InfoText;

/// Setup the 3D scene
fn setup_scene(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GraphCamera,
        CameraController {
            default_position: Vec3::new(0.0, 15.0, 20.0),
            default_target: Vec3::ZERO,
        },
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    // Info text
    commands.spawn((
        Text::new("CIM Graph Demo\nNodes: 0\nEdges: 0\n\nPress SPACE to add nodes\nClick nodes to select\nPress D to delete selected"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InfoText,
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(Plane3d::default().mesh().size(50.0, 50.0).build()),
        MeshMaterial3d(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            ..default()
        }),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
}

/// Create an initial demo graph
fn create_demo_graph(
    mut demo_state: ResMut<DemoState>,
    mut create_node: EventWriter<CreateNodeVisual>,
    mut create_edge: EventWriter<CreateEdgeVisual>,
) {
    let graph_id = GraphId::new();
    demo_state.current_graph_id = Some(graph_id);

    // Create a small initial graph
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(-5.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, -5.0),
    ];

    let mut node_ids = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        let node_id = uuid::Uuid::new_v4();
        node_ids.push(node_id);

        create_node.send(CreateNodeVisual {
            node_id,
            position: *pos,
            label: format!("Node {i + 1}"),
        });
    }

    // Create edges from center to periphery
    for i in 1..node_ids.len() {
        create_edge.send(CreateEdgeVisual {
            edge_id: uuid::Uuid::new_v4(),
            source_node_id: node_ids[0],
            target_node_id: node_ids[i],
            relationship: EdgeRelationship::Custom("Connection".to_string()),
        });
    }
}

/// Handle node creation events
fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<VisualNodeCreated>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeEntityMap>,
    mut demo_state: ResMut<DemoState>,
) {
    for event in create_events.read() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.7, 0.3),
            metallic: 0.3,
            perceptual_roughness: 0.7,
            ..default()
        });

        let entity = commands.spawn((
            NodeVisualBundle::new(event.node_id, GraphId::new(), event.position),
            Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
            MeshMaterial3d(material.clone()),
            NodeMaterial(material),
            AnimatedTransition {
                start_position: event.position + Vec3::Y * 10.0,
                target_position: event.position,
                progress: 0.0,
                duration: 1.0,
            },
        )).id();

        node_map.map.insert(event.node_id, entity);
        demo_state.node_count += 1;
    }
}

/// Handle edge creation events
fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<VisualEdgeCreated>,
    mut demo_state: ResMut<DemoState>,
) {
    for event in create_events.read() {
        commands.spawn(
            EdgeVisualBundle::new(
                event.edge_id,
                GraphId::new(),
                event.source_entity,
                event.target_entity,
            )
        );
        demo_state.edge_count += 1;
    }
}

/// Handle mouse interaction
fn handle_mouse_interaction(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &NodeVisual, &Transform, &NodeMaterial)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut demo_state: ResMut<DemoState>,
    mut node_click_events: EventWriter<NodeClicked>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = camera.get_single() else { return };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let mut hovered_node = None;

            // Check for node intersection
            for (entity, node_visual, transform, node_material) in nodes.iter() {
                let sphere_center = transform.translation;
                let sphere_radius = 0.6;

                let ray_origin = ray.origin;
                let ray_direction = ray.direction.as_vec3();
                let to_sphere = sphere_center - ray_origin;
                let t = to_sphere.dot(ray_direction).max(0.0);
                let closest_point = ray_origin + ray_direction * t;
                let distance = (closest_point - sphere_center).length();

                if distance < sphere_radius {
                    hovered_node = Some(node_visual.node_id);

                    // Update material for hover effect
                    if let Some(material) = materials.get_mut(&node_material.0) {
                        if demo_state.hovering_node != hovered_node {
                            material.base_color = Color::srgb(0.4, 0.8, 0.4);
                        }
                    }

                    // Handle click
                    if mouse_button.just_pressed(MouseButton::Left) {
                        demo_state.selected_node = Some(node_visual.node_id);

                        node_click_events.send(NodeClicked {
                            entity,
                            node_id: node_visual.node_id,
                        });

                        // Update material for selection
                        if let Some(material) = materials.get_mut(&node_material.0) {
                            material.base_color = Color::srgb(0.8, 0.4, 0.4);
                        }
                    }

                    break;
                }
            }

            // Reset hover state for non-hovered nodes
            if demo_state.hovering_node != hovered_node {
                if let Some(old_node_id) = demo_state.hovering_node {
                    if let Some(&entity) = node_map.map.get(&old_node_id) {
                        if let Ok((_, _, _, node_material)) = nodes.get(entity) {
                            if let Some(material) = materials.get_mut(&node_material.0) {
                                if demo_state.selected_node != Some(old_node_id) {
                                    material.base_color = Color::srgb(0.3, 0.7, 0.3);
                                }
                            }
                        }
                    }
                }
                demo_state.hovering_node = hovered_node;
            }
        }
    }
}

/// Handle keyboard input
fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    mut create_node: EventWriter<CreateNodeVisual>,
    mut remove_node: EventWriter<RemoveNodeVisual>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Add a new node at a random position
        let pos = Vec3::new(
            (rand::random::<f32>() - 0.5) * 20.0,
            0.0,
            (rand::random::<f32>() - 0.5) * 20.0,
        );

        create_node.send(CreateNodeVisual {
            node_id: uuid::Uuid::new_v4(),
            position: pos,
            label: "Dynamic Node".to_string(),
        });
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        // Delete selected node
        if let Some(node_id) = demo_state.selected_node {
            remove_node.send(RemoveNodeVisual {
                node_id,
            });
            demo_state.selected_node = None;
        }
    }
}

/// Animate nodes dropping in
fn animate_nodes(
    mut query: Query<(&mut Transform, &mut AnimatedTransition)>,
    time: Res<Time>,
) {
    for (mut transform, mut transition) in query.iter_mut() {
        if transition.progress < 1.0 {
            transition.progress += time.delta_secs() / transition.duration;
            transition.progress = transition.progress.min(1.0);

            // Smooth easing
            let t = ease_out_cubic(transition.progress);
            transform.translation = transition.start_position.lerp(transition.target_position, t);
        }
    }
}

/// Render edges
fn render_edges(
    mut gizmos: Gizmos,
    edges: Query<&EdgeVisual>,
    nodes: Query<&Transform>,
) {
    for edge in edges.iter() {
        if let (Ok(source_transform), Ok(target_transform)) = (
            nodes.get(edge.source_entity),
            nodes.get(edge.target_entity),
        ) {
            gizmos.line(
                source_transform.translation,
                target_transform.translation,
                Color::srgb(0.6, 0.6, 0.6),
            );
        }
    }
}

/// Update info text
fn update_info_text(
    mut text_query: Query<&mut Text, With<InfoText>>,
    demo_state: Res<DemoState>,
) {
    if demo_state.is_changed() {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.0 = format!("CIM Graph Demo\n\
                Nodes: {demo_state.node_count}\n\
                Edges: {demo_state.edge_count}\n\
                Selected: {if demo_state.selected_node.is_some(}\n\n\
                Press SPACE to add nodes\n\
                Click nodes to select\n\
                Press D to delete selected") { "Yes" } else { "No" }
            );
        }
    }
}

/// Easing function for smooth animation
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

// Add rand for demo purposes
mod rand {
    pub fn random<T>() -> T
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        use rand::Rng;
        rand::thread_rng().gen()
    }
}
