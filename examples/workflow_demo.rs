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
    println!("Starting Workflow Demo - Limited features version");
    println!("This demo visualizes a simple workflow with nodes and edges");
    
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(CimVizPlugin::default())
        .insert_resource(WorkflowDemo::default())
        .insert_resource(NodeEntityMap::default())
        .add_systems(Startup, create_workflow)
        .add_systems(
            Update,
            (
                handle_node_creation,
                handle_edge_creation,
                print_workflow_status,
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

fn create_workflow(
    mut demo: ResMut<WorkflowDemo>,
    mut create_node: EventWriter<CreateNodeVisual>,
    mut create_edge: EventWriter<CreateEdgeVisual>,
) {
    demo.graph_id = GraphId::new();
    
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

        demo.node_states.insert(node_id, NodeState::Pending);

        println!("Creating node: {} ({:?})", name, node_type);
        
        create_node.write(CreateNodeVisual {
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
        
        create_edge.write(CreateEdgeVisual {
            edge_id: EdgeId::new(),
            source_node_id: node_ids[from],
            target_node_id: node_ids[to],
            relationship: EdgeRelationship::Custom(label.to_string()),
        });
    }
    
    println!("\n=== Workflow Created Successfully ===\n");
}

fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeVisual>,
    mut node_map: ResMut<NodeEntityMap>,
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

        let entity = commands
            .spawn((
                NodeVisualBundle::new(event.node_id, demo.graph_id, event.position),
                WorkflowNode {
                    node_type,
                    state: NodeState::Pending,
                    label: event.label.clone(),
                },
            ))
            .id();

        node_map.map.insert(event.node_id, entity);
        
        println!("Created visual for node: {} at position {:?}", event.label, event.position);
    }
}

fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeVisual>,
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
            
            commands.spawn((
                EdgeVisualBundle::new(event.edge_id, demo.graph_id, source_entity, target_entity),
                EdgeLine { label: label.clone() },
            ));
            
            println!("Created edge: {}", label);
        }
    }
}

fn print_workflow_status(
    demo: Res<WorkflowDemo>,
    nodes: Query<&WorkflowNode>,
    edges: Query<&EdgeLine>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    
    // Print status every 60 frames (approximately once per second at 60 FPS)
    if *frame_count % 60 == 0 {
        let node_count = nodes.iter().count();
        let edge_count = edges.iter().count();
        
        if node_count > 0 || edge_count > 0 {
            println!("\n--- Workflow Status (Frame {}) ---", *frame_count);
            println!("State: {:?}", demo.workflow_state);
            println!("Nodes: {}", node_count);
            println!("Edges: {}", edge_count);
            
            if node_count > 0 {
                println!("\nNodes in workflow:");
                for node in nodes.iter() {
                    println!("  - {} ({:?}, {:?})", node.label, node.node_type, node.state);
                }
            }
        }
    }
}
