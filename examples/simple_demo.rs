//! Simple Demo of CIM-Domain-Bevy
//!
//! A minimal example showing how to visualize a graph with cim-domain-bevy.
//!
//! Run with: cargo run --example simple_demo --package cim-domain-bevy

use bevy::prelude::*;
use cim_contextgraph::{ContextGraphId as GraphId, EdgeId, NodeId};
use cim_domain_bevy::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .insert_resource(NodeMap::default())
        .insert_resource(GraphCreated(false))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_node_creation,
                handle_edge_creation,
                render_edges,
                create_initial_graph.run_if(resource_equals(GraphCreated(false))),
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct NodeMap {
    nodes: HashMap<NodeId, Entity>,
}

#[derive(Resource, PartialEq)]
struct GraphCreated(bool);

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Info text
    commands.spawn((
        Text::new("Simple CIM Graph Demo\nNodes and edges will appear automatically"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn create_initial_graph(
    mut create_node: EventWriter<CreateNodeVisual>,
    mut create_edge: EventWriter<CreateEdgeVisual>,
    mut created: ResMut<GraphCreated>,
) {
    created.0 = true;

    // Create a simple triangle of nodes
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();

    // Send node creation events
    create_node.send(CreateNodeVisual {
        node_id: node1,
        position: Vec3::new(-3.0, 0.0, 0.0),
        label: "Node 1".to_string(),
    });

    create_node.send(CreateNodeVisual {
        node_id: node2,
        position: Vec3::new(3.0, 0.0, 0.0),
        label: "Node 2".to_string(),
    });

    create_node.send(CreateNodeVisual {
        node_id: node3,
        position: Vec3::new(0.0, 0.0, -3.0),
        label: "Node 3".to_string(),
    });

    // Create edges
    create_edge.send(CreateEdgeVisual {
        edge_id: EdgeId::new(),
        source_node_id: node1,
        target_node_id: node2,
        relationship: EdgeRelationship::Custom("connects".to_string()),
    });

    create_edge.send(CreateEdgeVisual {
        edge_id: EdgeId::new(),
        source_node_id: node2,
        target_node_id: node3,
        relationship: EdgeRelationship::Custom("connects".to_string()),
    });

    create_edge.send(CreateEdgeVisual {
        edge_id: EdgeId::new(),
        source_node_id: node3,
        target_node_id: node1,
        relationship: EdgeRelationship::Custom("connects".to_string()),
    });

    info!("Created initial graph with 3 nodes and 3 edges");
}

fn handle_node_creation(
    mut commands: Commands,
    mut events: EventReader<VisualNodeCreated>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeMap>,
) {
    for event in events.read() {
        info!(
            "Creating visual for node {:?} at {:?}",
            event.node_id, event.position
        );

        let entity = commands
            .spawn((
                NodeVisualBundle::new(event.node_id, GraphId::new(), event.position),
                Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.7, 0.9),
                    ..default()
                })),
            ))
            .id();

        node_map.nodes.insert(event.node_id, entity);
    }
}

fn handle_edge_creation(
    mut commands: Commands,
    mut events: EventReader<VisualEdgeCreated>,
    node_map: Res<NodeMap>,
) {
    for event in events.read() {
        info!(
            "Creating edge visual between source entity {:?} and target entity {:?}",
            event.source_entity, event.target_entity
        );

        commands.spawn(EdgeVisualBundle::new(
            event.edge_id,
            GraphId::new(),
            event.source_entity,
            event.target_entity,
        ));
    }
}

fn render_edges(mut gizmos: Gizmos, edges: Query<&EdgeVisual>, nodes: Query<&Transform>) {
    for edge in edges.iter() {
        if let (Ok(source_transform), Ok(target_transform)) =
            (nodes.get(edge.source_entity), nodes.get(edge.target_entity))
        {
            gizmos.line(
                source_transform.translation,
                target_transform.translation,
                Color::srgb(0.6, 0.6, 0.6),
            );
        }
    }
}
