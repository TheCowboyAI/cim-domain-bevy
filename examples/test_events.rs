//! Simple test to verify event logging is working

use bevy::prelude::*;
use cim_domain_bevy::*;

fn main() {
    println!("Starting event test demo...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, test_events)
        .run();
}

fn setup(mut commands: Commands, mut create_node: EventWriter<CreateNodeVisual>) {
    println!("Setup: Creating test node");

    // Create a test node visual
    create_node.send(CreateNodeVisual {
        node_id: uuid::Uuid::new_v4(),
        position: Vec3::ZERO,
        label: "Test Node".to_string(),
    });
}

fn test_events(
    mut node_created: EventReader<VisualNodeCreated>,
    mut node_clicked: EventReader<NodeClicked>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut create_node: EventWriter<CreateNodeVisual>,
) {
    for event in node_created.read() {
        println!("Node visual created: {:?}", event.node_id);
    }

    for event in node_clicked.read() {
        println!("Node clicked: {:?}", event.node_id);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        println!("SPACE pressed - creating test node");

        create_node.send(CreateNodeVisual {
            node_id: uuid::Uuid::new_v4(),
            position: Vec3::new(1.0, 2.0, 3.0),
            label: "Dynamic Node".to_string(),
        });
    }
}
