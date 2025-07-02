//! Basic ECS Example
//!
//! This example demonstrates:
//! - Basic Bevy ECS setup
//! - Component creation
//! - System implementation
//! - Resource management

use bevy::prelude::*;
use cim_domain_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};

fn main() {
    println!("=== CIM Bevy Domain Example ===\n");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .insert_resource(Selection::default())
        .insert_resource(VisualizationConfig::default())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_selection)
        .run();
}

fn setup(
    mut commands: Commands,
    mut create_node: EventWriter<CreateNodeVisual>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GraphCamera,
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Create a sample node
    create_node.send(CreateNodeVisual {
        node_id: uuid::Uuid::new_v4(),
        position: Vec3::ZERO,
        label: "Sample Node".to_string(),
    });
}

fn handle_selection(
    mut node_clicked: EventReader<NodeClicked>,
    mut selection: ResMut<Selection>,
) {
    for event in node_clicked.read() {
        println!("Node clicked: {:?}", event.node_id);
        selection.nodes.push((event.entity, event.node_id));
    }
}
