//! Fixed Workflow Demo with proper shader setup
//!
//! This version ensures shaders compile correctly

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_nodes)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Additional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.3)),
        ..default()
    });

    // Ground - using a cube instead of plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(20.0, 0.1, 20.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });

    println!("Creating workflow visualization...");

    // Workflow nodes
    let positions = vec![
        ("Start", Vec3::new(-6.0, 0.0, 0.0), Color::srgb(0.0, 1.0, 0.0)),
        ("Submit", Vec3::new(-3.0, 0.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Review", Vec3::new(0.0, 0.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Decision", Vec3::new(3.0, 0.0, 0.0), Color::srgb(0.8, 0.8, 0.0)),
        ("Approved", Vec3::new(6.0, 0.0, 0.0), Color::srgb(0.0, 0.8, 0.0)),
    ];

    // Create nodes
    for (i, (name, pos, color)) in positions.iter().enumerate() {
        let mesh = if i == 0 || i == 4 {
            meshes.add(Sphere::new(0.5))
        } else if i == 3 {
            meshes.add(Cuboid::new(1.0, 1.0, 1.0))
        } else {
            meshes.add(Cylinder::new(0.5, 1.0))
        };

        commands.spawn((
            PbrBundle {
                mesh,
                material: materials.add(StandardMaterial {
                    base_color: *color,
                    ..default()
                }),
                transform: Transform::from_translation(*pos),
                ..default()
            },
            WorkflowNode,
        ));

        // Add text label above node
        println!("Created node: {} at {:?}", name, pos);
    }

    // Create edges as thin cylinders
    let edges = vec![
        (Vec3::new(-6.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 0.0)),
        (Vec3::new(-3.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0)),
        (Vec3::new(3.0, 0.0, 0.0), Vec3::new(6.0, 0.0, 0.0)),
    ];

    for (start, end) in edges {
        let midpoint = (start + end) / 2.0;
        let diff = end - start;
        let length = diff.length();

        commands.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.05, length)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.4, 0.4),
                ..default()
            }),
            transform: Transform::from_translation(midpoint)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
            ..default()
        });
    }

    println!("\nWorkflow visualization complete!");
    println!("You should see:");
    println!("- Green sphere (Start)");
    println!("- Gray cylinders (Submit, Review)");
    println!("- Yellow cube (Decision)");
    println!("- Green sphere (Approved)");
    println!("- Gray lines connecting them");
}

#[derive(Component)]
struct WorkflowNode;

fn rotate_nodes(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<WorkflowNode>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
} 