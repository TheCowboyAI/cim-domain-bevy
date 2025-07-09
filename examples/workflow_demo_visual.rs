//! Visual Workflow Demo - Complete
//!
//! A proper workflow visualization with nodes, edges, and animations.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.3,
            affects_lightmapped_meshes: false,
        })
        .insert_resource(WorkflowState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            rotate_nodes,
            animate_workflow,
            handle_input,
        ))
        .run();
}

#[derive(Resource, Default)]
struct WorkflowState {
    current_step: usize,
    timer: Timer,
    is_running: bool,
}

impl WorkflowState {
    fn new() -> Self {
        Self {
            current_step: 0,
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            is_running: false,
        }
    }
}

#[derive(Component)]
struct WorkflowNode {
    index: usize,
    name: String,
}

#[derive(Component)]
struct WorkflowEdge;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with better angle
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Add some directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    println!("\n=== WORKFLOW VISUALIZATION ===");
    println!("Creating workflow nodes...");

    // Define workflow nodes with better layout
    let nodes = vec![
        ("Start", Vec3::new(-8.0, 0.0, 0.0), NodeType::Start),
        ("Submit", Vec3::new(-4.0, 0.0, 0.0), NodeType::Task),
        ("Review", Vec3::new(0.0, 0.0, 0.0), NodeType::Task),
        ("Decision", Vec3::new(4.0, 0.0, 0.0), NodeType::Decision),
        ("Approved", Vec3::new(8.0, 0.0, 2.0), NodeType::End),
        ("Rejected", Vec3::new(8.0, 0.0, -2.0), NodeType::End),
    ];

    // Create nodes
    for (i, (name, position, node_type)) in nodes.iter().enumerate() {
        let (mesh, color) = match node_type {
            NodeType::Start => (
                meshes.add(Sphere::new(0.8)),
                Color::srgb(0.2, 0.8, 0.2),
            ),
            NodeType::Task => (
                meshes.add(Cylinder::new(0.8, 1.2)),
                Color::srgb(0.3, 0.5, 0.8),
            ),
            NodeType::Decision => (
                meshes.add(Cuboid::new(1.4, 1.4, 1.4)),
                Color::srgb(0.8, 0.8, 0.2),
            ),
            NodeType::End => (
                meshes.add(Sphere::new(0.8)),
                if name == &"Approved" {
                    Color::srgb(0.2, 0.8, 0.2)
                } else {
                    Color::srgb(0.8, 0.2, 0.2)
                },
            ),
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::from(color) * 0.02,
                perceptual_roughness: 0.8,
                metallic: 0.1,
                ..default()
            })),
            Transform::from_translation(*position),
            WorkflowNode {
                index: i,
                name: name.to_string(),
            },
        ));
        
        println!("  {} {} at {:?}", 
            match node_type {
                NodeType::Start => "🟢",
                NodeType::Task => "🔵",
                NodeType::Decision => "🟡",
                NodeType::End => if name == &"Approved" { "✅" } else { "❌" },
            },
            name, 
            position
        );
    }

    // Create edges
    let edges = vec![
        (nodes[0].1, nodes[1].1), // Start -> Submit
        (nodes[1].1, nodes[2].1), // Submit -> Review
        (nodes[2].1, nodes[3].1), // Review -> Decision
        (nodes[3].1, nodes[4].1), // Decision -> Approved
        (nodes[3].1, nodes[5].1), // Decision -> Rejected
    ];

    println!("\nCreating connections...");
    for (start, end) in edges {
        create_edge(&mut commands, &mut meshes, &mut materials, start, end);
    }

    println!("\n=== CONTROLS ===");
    println!("SPACE - Start/pause animation");
    println!("R     - Reset workflow");
    println!("================\n");
}

fn create_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,
) {
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    let length = start.distance(end);
    
    // Create edge as a thin cylinder
    let edge_mesh = meshes.add(Cylinder::new(0.05, length));
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.5),
        emissive: LinearRgba::from(Color::srgb(0.1, 0.1, 0.15)),
        metallic: 0.3,
        perceptual_roughness: 0.7,
        ..default()
    });
    
    // Calculate rotation to align cylinder with edge direction
    let up = if direction.y.abs() > 0.99 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    
    let rotation = Transform::from_translation(midpoint)
        .looking_at(start, up)
        .rotation * Quat::from_rotation_x(std::f32::consts::PI / 2.0);
    
    commands.spawn((
        Mesh3d(edge_mesh),
        MeshMaterial3d(edge_material),
        Transform::from_translation(midpoint).with_rotation(rotation),
        WorkflowEdge,
    ));
}

fn rotate_nodes(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<WorkflowNode>>,
) {
    for mut transform in query.iter_mut() {
        // Gentle rotation
        transform.rotate_y(time.delta_secs() * 0.3);
    }
}

fn animate_workflow(
    time: Res<Time>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
) {
    if !workflow.is_running {
        return;
    }

    workflow.timer.tick(time.delta());

    if workflow.timer.just_finished() {
        // Reset previous node color
        for (node, material_handle) in nodes.iter() {
            if node.index == workflow.current_step {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    // Restore original color
                    material.emissive = LinearRgba::from(material.base_color) * 0.02;
                }
            }
        }

        // Move to next step
        workflow.current_step = (workflow.current_step + 1) % 6;

        // Highlight current node
        for (node, material_handle) in nodes.iter() {
            if node.index == workflow.current_step {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    // Make it glow
                    material.emissive = LinearRgba::from(material.base_color) * 0.5;
                    println!("➡️  Active: {}", node.name);
                }
            }
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        workflow.is_running = !workflow.is_running;
        println!("Animation: {}", if workflow.is_running { "▶️  RUNNING" } else { "⏸️  PAUSED" });
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        workflow.is_running = false;
        workflow.current_step = 0;
        workflow.timer.reset();

        // Reset all materials
        for (_, material_handle) in nodes.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.emissive = LinearRgba::from(material.base_color) * 0.02;
            }
        }

        println!("🔄 Workflow reset!");
    }
}

#[derive(Clone, Copy)]
enum NodeType {
    Start,
    Task,
    Decision,
    End,
} 