//! Workflow Visualization Demo
//!
//! This demo shows how to visualize a business workflow using cim-domain-bevy.
//! It demonstrates a document approval workflow with different node types
//! and animated state transitions.
//!
//! Run with: cargo run --example workflow_demo

use bevy::prelude::*;
use cim_domain_bevy::*;
use std::collections::HashMap;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};

fn main() {
    println!("Starting Workflow Visualization Demo");
    println!("This demo visualizes a document approval workflow with interactive nodes and edges");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM Workflow Visualization Demo".into(),
                resolution: (1200., 800.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CimVizPlugin::default())
        .insert_resource(WorkflowDemo::default())
        .insert_resource(NodeEntityMap::default())
        .add_systems(Startup, (setup_scene, create_workflow))
        .add_systems(
            Update,
            (
                handle_node_creation,
                handle_edge_creation,
                animate_workflow,
                update_node_visuals,
                handle_input,
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct WorkflowDemo {
    graph_id: GraphId,
    workflow_state: WorkflowState,
    current_step: usize,
    node_states: HashMap<NodeId, NodeState>,
    animation_timer: Timer,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum WorkflowState {
    #[default]
    NotStarted,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum NodeState {
    Pending,
    Active,
    Completed,
    Failed,
}

#[derive(Component)]
struct WorkflowNode {
    node_type: WorkflowNodeType,
    state: NodeState,
    label: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum WorkflowNodeType {
    Start,
    Task,
    Decision,
    End,
}

#[derive(Resource, Default)]
struct NodeEntityMap {
    map: HashMap<NodeId, Entity>,
}

#[derive(Component)]
struct EdgeLine {
    label: String,
}

// Component to track material for state changes
#[derive(Component)]
struct NodeMaterial {
    state: NodeState,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.3)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    println!("Scene setup complete");
}

fn create_workflow(
    mut demo: ResMut<WorkflowDemo>,
    mut create_node: EventWriter<CreateNodeVisual>,
    mut create_edge: EventWriter<CreateEdgeVisual>,
) {
    demo.graph_id = GraphId::new();
    demo.animation_timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    
    println!("\n=== Creating Document Approval Workflow ===");
    
    // Define workflow nodes
    let nodes = vec![
        ("Start", WorkflowNodeType::Start, Vec3::new(-8.0, 0.0, 0.0)),
        ("Submit Document", WorkflowNodeType::Task, Vec3::new(-4.0, 0.0, 0.0)),
        ("Review", WorkflowNodeType::Task, Vec3::new(0.0, 0.0, 0.0)),
        ("Approval Decision", WorkflowNodeType::Decision, Vec3::new(4.0, 0.0, 0.0)),
        ("Revise", WorkflowNodeType::Task, Vec3::new(0.0, -4.0, 0.0)),
        ("Approved", WorkflowNodeType::End, Vec3::new(8.0, 0.0, 0.0)),
        ("Rejected", WorkflowNodeType::End, Vec3::new(8.0, -4.0, 0.0)),
    ];

    let mut node_ids = Vec::new();

    // Create nodes
    for (name, node_type, position) in nodes {
        let node_id = NodeId::new();
        node_ids.push(node_id);

        let initial_state = if node_type == WorkflowNodeType::Start {
            NodeState::Completed
        } else {
            NodeState::Pending
        };

        demo.node_states.insert(node_id, initial_state);

        println!("Creating node: {} ({:?})", name, node_type);
        
        create_node.send(CreateNodeVisual {
            node_id,
            position,
            label: name.to_string(),
        });
    }

    // Create edges
    let edges = vec![
        (0, 1, "Start Process"),
        (1, 2, "Submit"),
        (2, 3, "Review Complete"),
        (3, 5, "Approve"),
        (3, 4, "Needs Revision"),
        (3, 6, "Reject"),
        (4, 2, "Resubmit"),
    ];

    println!("\nCreating workflow edges:");
    for (from, to, label) in edges {
        println!("  {} -> {} ({})", from, to, label);
        
        create_edge.send(CreateEdgeVisual {
            edge_id: EdgeId::new(),
            source_node_id: node_ids[from],
            target_node_id: node_ids[to],
            relationship: EdgeRelationship::Custom(label.to_string()),
        });
    }
    
    demo.workflow_state = WorkflowState::Running;
    println!("\n=== Workflow Created Successfully ===\n");
}

fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeVisual>,
    mut node_map: ResMut<NodeEntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    demo: Res<WorkflowDemo>,
) {
    for event in create_events.read() {
        // Parse node type from position
        let node_type = match event.position.x {
            x if x < -6.0 => WorkflowNodeType::Start,
            x if x > 6.0 => WorkflowNodeType::End,
            x if x > 2.0 && x < 6.0 => WorkflowNodeType::Decision,
            _ => WorkflowNodeType::Task,
        };

        let state = demo.node_states.get(&event.node_id).copied().unwrap_or(NodeState::Pending);
        
        // Create visual representation
        let mesh = match node_type {
            WorkflowNodeType::Start | WorkflowNodeType::End => {
                meshes.add(Sphere::new(0.5))
            }
            WorkflowNodeType::Decision => {
                meshes.add(Cuboid::new(1.0, 1.0, 1.0))
            }
            WorkflowNodeType::Task => {
                meshes.add(Cylinder::new(0.5, 0.8))
            }
        };

        let material = materials.add(StandardMaterial {
            base_color: get_color_for_state(state),
            ..default()
        });

        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                NodeVisualBundle::new(event.node_id, demo.graph_id, event.position),
                WorkflowNode {
                    node_type,
                    state,
                    label: event.label.clone(),
                },
                NodeMaterial { state },
            ))
            .id();

        node_map.map.insert(event.node_id, entity);
        
        println!("Created visual for node: {} at position {:?}", event.label, event.position);
    }
}

fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_map: Res<NodeEntityMap>,
    demo: Res<WorkflowDemo>,
) {
    for event in create_events.read() {
        if let (Some(&source_entity), Some(&target_entity)) = (
            node_map.map.get(&event.source_node_id),
            node_map.map.get(&event.target_node_id),
        ) {
            let label = match &event.relationship {
                EdgeRelationship::Custom(s) => s.clone(),
                _ => "connects".to_string(),
            };
            
            // Create a simple line mesh (placeholder - in real implementation, use gizmos or custom mesh)
            let mesh = meshes.add(Cuboid::new(0.1, 0.1, 1.0));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            });
            
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                EdgeVisualBundle::new(event.edge_id, demo.graph_id, source_entity, target_entity),
                EdgeLine { label: label.clone() },
            ));
            
            println!("Created edge: {}", label);
        }
    }
}

fn animate_workflow(
    time: Res<Time>,
    mut demo: ResMut<WorkflowDemo>,
    mut node_query: Query<(&WorkflowNode, &mut NodeMaterial)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    demo.animation_timer.tick(time.delta());
    
    if demo.animation_timer.just_finished() && demo.workflow_state == WorkflowState::Running {
        // Progress through workflow steps
        let steps = vec![
            ("Submit Document", NodeState::Active),
            ("Submit Document", NodeState::Completed),
            ("Review", NodeState::Active),
            ("Review", NodeState::Completed),
            ("Approval Decision", NodeState::Active),
            ("Approved", NodeState::Active),
        ];
        
        if demo.current_step < steps.len() {
            let (label, new_state) = steps[demo.current_step];
            
            // Update node state
            for (node, mut node_material) in node_query.iter_mut() {
                if node.label == label {
                    node_material.state = new_state;
                    println!("Updated {} to {:?}", label, new_state);
                }
            }
            
            demo.current_step += 1;
            
            if demo.current_step >= steps.len() {
                demo.workflow_state = WorkflowState::Completed;
                println!("\n🎉 Workflow completed!");
            }
        }
    }
}

fn update_node_visuals(
    demo: Res<WorkflowDemo>,
    node_query: Query<(&NodeVisual, &NodeMaterial, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (node_visual, node_material, material_handle) in node_query.iter() {
        if let Some(&state) = demo.node_states.get(&node_visual.node_id) {
            if state != node_material.state {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color = get_color_for_state(state);
                }
            }
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<WorkflowDemo>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match demo.workflow_state {
            WorkflowState::NotStarted => {
                demo.workflow_state = WorkflowState::Running;
                println!("Workflow started!");
            }
            WorkflowState::Completed => {
                demo.workflow_state = WorkflowState::Running;
                demo.current_step = 0;
                println!("Workflow restarted!");
            }
            _ => {}
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        demo.current_step = 0;
        demo.workflow_state = WorkflowState::NotStarted;
        println!("Workflow reset!");
    }
}

fn get_color_for_state(state: NodeState) -> Color {
    match state {
        NodeState::Pending => Color::srgb(0.5, 0.5, 0.5),
        NodeState::Active => Color::srgb(1.0, 0.8, 0.0),
        NodeState::Completed => Color::srgb(0.0, 0.8, 0.0),
        NodeState::Failed => Color::srgb(0.8, 0.0, 0.0),
    }
}
