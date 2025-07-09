//! Visual Workflow Demo
//!
//! This demo shows a 3D visualization of a workflow with animated state changes.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WorkflowState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_workflow, handle_input))
        .run();
}

#[derive(Resource, Default)]
struct WorkflowState {
    nodes: Vec<Entity>,
    current_step: usize,
    timer: Timer,
    is_running: bool,
}

#[derive(Component)]
struct WorkflowNode {
    name: String,
    index: usize,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut workflow: ResMut<WorkflowState>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.3)),
    ));

    // Point light for better visibility
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    println!("Creating workflow nodes...");

    // Workflow nodes
    let nodes_data = vec![
        ("Start", Vec3::new(-8.0, 1.0, 0.0), Color::srgb(0.0, 0.8, 0.0)),
        ("Submit", Vec3::new(-4.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Review", Vec3::new(0.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Decision", Vec3::new(4.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Revise", Vec3::new(0.0, 1.0, -4.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Approved", Vec3::new(8.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Rejected", Vec3::new(8.0, 1.0, -4.0), Color::srgb(0.5, 0.5, 0.5)),
    ];

    // Create nodes
    for (i, (name, position, color)) in nodes_data.iter().enumerate() {
        let mesh = if i == 0 || i == 5 || i == 6 {
            // Start and end nodes are spheres
            meshes.add(Sphere::new(1.0))
        } else if i == 3 {
            // Decision node is a cube
            meshes.add(Cuboid::new(1.8, 1.8, 1.8))
        } else {
            // Task nodes are cylinders
            meshes.add(Cylinder::new(0.9, 1.8))
        };

        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                ..default()
            })),
            Transform::from_translation(*position),
            WorkflowNode {
                name: name.to_string(),
                index: i,
            },
        )).id();

        workflow.nodes.push(entity);
        println!("Created node: {} at {:?}", name, position);
    }

    // Create edges
    let edges = vec![
        (0, 1), // Start -> Submit
        (1, 2), // Submit -> Review
        (2, 3), // Review -> Decision
        (3, 5), // Decision -> Approved
        (3, 4), // Decision -> Revise
        (3, 6), // Decision -> Rejected
        (4, 2), // Revise -> Review
    ];

    for (from, to) in edges {
        let from_pos = nodes_data[from].1;
        let to_pos = nodes_data[to].1;
        let midpoint = (from_pos + to_pos) / 2.0;
        let diff = to_pos - from_pos;
        let length = diff.length();
        
        // Create edge visualization
        let edge_mesh = meshes.add(Cylinder::new(0.05, length));
        let edge_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        });
        
        // Calculate rotation
        let up = if diff.normalize().dot(Vec3::Y).abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        
        let rotation = Transform::from_translation(midpoint)
            .looking_at(from_pos, up)
            .rotation * Quat::from_rotation_x(std::f32::consts::PI / 2.0);
        
        commands.spawn((
            Mesh3d(edge_mesh),
            MeshMaterial3d(edge_material),
            Transform::from_translation(midpoint).with_rotation(rotation),
        ));
    }

    workflow.timer = Timer::from_seconds(1.5, TimerMode::Repeating);
    
    println!("\nWorkflow setup complete!");
    println!("\n=== CONTROLS ===");
    println!("Press SPACE to start/pause the animation");
    println!("Press R to reset");
    println!("================\n");
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        workflow.is_running = !workflow.is_running;
        println!("Animation: {}", if workflow.is_running { "RUNNING" } else { "PAUSED" });
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        workflow.current_step = 0;
        workflow.is_running = false;
        
        // Reset all node colors
        for (node, material_handle) in query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = if node.index == 0 {
                    Color::srgb(0.0, 0.8, 0.0) // Start node stays green
                } else {
                    Color::srgb(0.5, 0.5, 0.5) // Others go gray
                };
            }
        }
        
        println!("Workflow reset!");
    }
}

fn animate_workflow(
    time: Res<Time>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
) {
    if !workflow.is_running {
        return;
    }
    
    workflow.timer.tick(time.delta());
    
    if workflow.timer.just_finished() && workflow.current_step < 6 {
        let active_color = Color::srgb(1.0, 0.8, 0.0); // Yellow for active
        let completed_color = Color::srgb(0.0, 0.8, 0.0); // Green for completed
        
        // Update current node to active
        for (node, material_handle) in query.iter() {
            if node.index == workflow.current_step + 1 {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color = active_color;
                    println!("Activating: {}", node.name);
                }
            }
            // Mark previous node as completed
            else if node.index == workflow.current_step && node.index > 0 {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color = completed_color;
                }
            }
        }
        
        workflow.current_step += 1;
        
        if workflow.current_step >= 6 {
            // Mark final node as completed
            for (node, material_handle) in query.iter() {
                if node.index == 5 { // Approved node
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        material.base_color = completed_color;
                    }
                }
            }
            
            println!("\n🎉 Workflow completed!");
            workflow.is_running = false;
        }
    }
} 