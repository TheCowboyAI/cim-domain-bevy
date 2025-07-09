//! Minimal Workflow Demo - No shader issues
//!
//! This version uses the simplest possible rendering to avoid shader compilation problems.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
            affects_lightmapped_meshes: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_nodes)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - simple setup
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("Creating workflow nodes...");

    // Create simple colored cubes for workflow nodes
    let node_positions = vec![
        ("Start", Vec3::new(-6.0, 0.0, 0.0), Color::srgb(0.0, 1.0, 0.0)),
        ("Submit", Vec3::new(-3.0, 0.0, 0.0), Color::srgb(0.5, 0.5, 1.0)),
        ("Review", Vec3::new(0.0, 0.0, 0.0), Color::srgb(0.5, 0.5, 1.0)),
        ("Decision", Vec3::new(3.0, 0.0, 0.0), Color::srgb(1.0, 1.0, 0.0)),
        ("Approved", Vec3::new(6.0, 0.0, 0.0), Color::srgb(0.0, 1.0, 0.0)),
    ];

    // Create nodes as simple cubes
    for (name, position, color) in node_positions.iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                unlit: true, // Use unlit to avoid complex shaders
                ..default()
            })),
            Transform::from_translation(*position),
            WorkflowNode,
        ));
        
        println!("Created node: {} at {:?}", name, position);
    }

    // Create edges as thin rectangles
    let edges = vec![
        (Vec3::new(-6.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 0.0)),
        (Vec3::new(-3.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)),
        (Vec3::new(3.0, 0.0, 0.0), Vec3::new(6.0, 0.0, 0.0)),
    ];

    for (start, end) in edges {
        let midpoint = (start + end) / 2.0;
        let length = (end - start).length();
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(length, 0.1, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.7, 0.7),
                unlit: true, // Use unlit to avoid complex shaders
                ..default()
            })),
            Transform::from_translation(midpoint),
        ));
    }

    println!("\nWorkflow visualization ready!");
    println!("You should see:");
    println!("- Green cube (Start)");
    println!("- Blue cubes (Submit, Review)");
    println!("- Yellow cube (Decision)");
    println!("- Green cube (Approved)");
    println!("- Gray lines connecting them");
}

#[derive(Component)]
struct WorkflowNode;

fn rotate_nodes(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<WorkflowNode>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
} 