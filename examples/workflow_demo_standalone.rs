//! Standalone Workflow Visualization Demo
//!
//! This demo shows the workflow visualization without using CimVizPlugin
//! to isolate rendering issues.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Workflow Demo - Standalone".into(),
                resolution: (1200., 800.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(WorkflowState::default())
        .add_systems(Startup, setup_workflow)
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

fn setup_workflow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut workflow: ResMut<WorkflowState>,
) {
    // Camera - positioned to see the workflow better
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Add more lights for better visibility
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
        affects_lightmapped_meshes: false,
    });

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

    // Define workflow nodes - raised above the ground
    let nodes = vec![
        ("Start", Vec3::new(-8.0, 1.0, 0.0), Color::srgb(0.0, 0.8, 0.0)),
        ("Submit", Vec3::new(-4.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Review", Vec3::new(0.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Decision", Vec3::new(4.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Revise", Vec3::new(0.0, 1.0, -4.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Approved", Vec3::new(8.0, 1.0, 0.0), Color::srgb(0.5, 0.5, 0.5)),
        ("Rejected", Vec3::new(8.0, 1.0, -4.0), Color::srgb(0.5, 0.5, 0.5)),
    ];

    // Create nodes - make them bigger
    for (i, (name, position, color)) in nodes.iter().enumerate() {
        let shape = if i == 0 || i == 5 || i == 6 {
            // Start and end nodes are spheres
            meshes.add(Sphere::new(1.2))
        } else if i == 3 {
            // Decision node is a cube
            meshes.add(Cuboid::new(2.0, 2.0, 2.0))
        } else {
            // Task nodes are cylinders
            meshes.add(Cylinder::new(1.0, 2.0))
        };

        let entity = commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                metallic: 0.0,
                perceptual_roughness: 0.8,
                ..default()
            })),
            Transform::from_translation(*position),
        )).id();

        workflow.nodes.push(entity);
        println!("Created node: {} at {:?}", name, position);
    }

    // Create simple edge lines using thin cylinders
    let edge_connections = vec![
        (0, 1), // Start -> Submit
        (1, 2), // Submit -> Review
        (2, 3), // Review -> Decision
        (3, 5), // Decision -> Approved
        (3, 4), // Decision -> Revise
        (3, 6), // Decision -> Rejected
        (4, 2), // Revise -> Review
    ];

    for (from, to) in edge_connections {
        let from_pos = nodes[from].1;
        let to_pos = nodes[to].1;
        let midpoint = (from_pos + to_pos) / 2.0;
        let direction = to_pos - from_pos;
        let length = direction.length();

        // Calculate rotation to align cylinder with the edge direction
        let up = if direction.normalize().dot(Vec3::Y).abs() > 0.99 {
            Vec3::X
        } else {
            Vec3::Y
        };
        
        let rotation = Transform::from_translation(midpoint)
            .looking_at(from_pos, up)
            .rotation;

        // Create a thin cylinder as an edge
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.1, length))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.4, 0.4),
                metallic: 0.3,
                perceptual_roughness: 0.6,
                ..default()
            })),
            Transform::from_translation(midpoint)
                .with_rotation(rotation * Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
        ));
        
        println!("Created edge from node {} to node {}", from, to);
    }

    workflow.timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    println!("Workflow setup complete!");
    println!("\nPress SPACE to start the animation");
    println!("Press R to reset");
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workflow: ResMut<WorkflowState>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        workflow.is_running = !workflow.is_running;
        println!("Workflow is now: {}", if workflow.is_running { "RUNNING" } else { "PAUSED" });
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        workflow.current_step = 0;
        workflow.is_running = false;
        println!("Workflow reset!");
    }
}

fn animate_workflow(
    time: Res<Time>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    if !workflow.is_running {
        return;
    }
    
    workflow.timer.tick(time.delta());

    if workflow.timer.just_finished() && workflow.current_step < 6 {
        println!("Animating step {}", workflow.current_step);

        // Update colors based on workflow progress
        let active_color = Color::srgb(1.0, 0.8, 0.0); // Yellow for active
        let completed_color = Color::srgb(0.0, 0.8, 0.0); // Green for completed

        // Progress through the workflow
        if workflow.current_step < workflow.nodes.len() {
            if let Ok(material_handle) = query.get(workflow.nodes[workflow.current_step]) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color = active_color;
                    println!("Set node {} to active (yellow)", workflow.current_step);
                }
            }

            // Mark previous step as completed
            if workflow.current_step > 0 {
                let prev_entity = workflow.nodes[workflow.current_step - 1];
                if let Ok(material_handle) = query.get(prev_entity) {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        material.base_color = completed_color;
                        println!("Set node {} to completed (green)", workflow.current_step - 1);
                    }
                }
            }
        }

        workflow.current_step += 1;

        if workflow.current_step >= 6 {
            println!("🎉 Workflow animation complete!");
            workflow.is_running = false;
        }
    }
} 