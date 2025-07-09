//! Workflow Demo with UI Descriptions
//!
//! Shows node descriptions and workflow state in a UI overlay.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

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
        .insert_resource(CameraState::default())
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(Update, (
            camera_controls,
            rotate_nodes,
            animate_workflow,
            update_ui,
            handle_input,
        ))
        .run();
}

#[derive(Resource)]
struct WorkflowState {
    current_step: usize,
    timer: Timer,
    is_running: bool,
    nodes: Vec<NodeInfo>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            current_step: 0,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            is_running: false,
            nodes: vec![
                NodeInfo {
                    name: "Start".to_string(),
                    description: "Workflow begins when a document is uploaded to the system".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Submit".to_string(),
                    description: "Document is submitted for review with metadata and categorization".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Review".to_string(),
                    description: "Automated and manual review processes check document compliance".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Decision".to_string(),
                    description: "AI-powered decision engine evaluates document based on business rules".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Approved".to_string(),
                    description: "Document approved! Notification sent and document archived".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Rejected".to_string(),
                    description: "Document rejected. Reasons provided and sender notified".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Revise".to_string(),
                    description: "Document sent back for revision with specific feedback".to_string(),
                    status: NodeStatus::Pending,
                },
            ],
        }
    }
}

#[derive(Clone)]
struct NodeInfo {
    name: String,
    description: String,
    status: NodeStatus,
}

#[derive(Clone, Copy, PartialEq)]
enum NodeStatus {
    Pending,
    Active,
    Completed,
}

#[derive(Resource)]
struct CameraState {
    distance: f32,
    rotation: f32,
    height: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            distance: 25.0,
            rotation: 0.0,
            height: 10.0,
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

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct NodeListText;

#[derive(Component)]
struct CurrentNodeText;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with controls
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Define workflow nodes with better layout
    let nodes = vec![
        ("Start", Vec3::new(-12.0, 0.0, 0.0), NodeType::Start),
        ("Submit", Vec3::new(-6.0, 0.0, 0.0), NodeType::Task),
        ("Review", Vec3::new(0.0, 0.0, 0.0), NodeType::Task),
        ("Decision", Vec3::new(6.0, 0.0, 0.0), NodeType::Decision),
        ("Approved", Vec3::new(12.0, 0.0, 3.0), NodeType::End),
        ("Rejected", Vec3::new(12.0, 0.0, -3.0), NodeType::End),
        ("Revise", Vec3::new(0.0, 0.0, -6.0), NodeType::Task),
    ];

    // Create nodes
    for (i, (name, position, node_type)) in nodes.iter().enumerate() {
        let (mesh, color) = match node_type {
            NodeType::Start => (
                meshes.add(Sphere::new(1.2)),
                Color::srgb(0.2, 0.8, 0.2),
            ),
            NodeType::Task => (
                meshes.add(Cylinder::new(1.0, 1.5)),
                Color::srgb(0.3, 0.5, 0.8),
            ),
            NodeType::Decision => (
                meshes.add(Cuboid::new(2.0, 2.0, 2.0)),
                Color::srgb(0.8, 0.8, 0.2),
            ),
            NodeType::End => (
                meshes.add(Sphere::new(1.2)),
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
    }

    // Create edges
    let edges = vec![
        (nodes[0].1, nodes[1].1), // Start -> Submit
        (nodes[1].1, nodes[2].1), // Submit -> Review
        (nodes[2].1, nodes[3].1), // Review -> Decision
        (nodes[3].1, nodes[4].1), // Decision -> Approved
        (nodes[3].1, nodes[5].1), // Decision -> Rejected
        (nodes[3].1, nodes[6].1), // Decision -> Revise
        (nodes[6].1, nodes[2].1), // Revise -> Review (loop back)
    ];

    for (start, end) in edges {
        create_edge(&mut commands, &mut meshes, &mut materials, start, end);
    }
}

fn setup_ui(mut commands: Commands) {
    // Root UI container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::NONE.into()),
    )).with_children(|parent| {
        // Top panel for current node info
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("📊 Document Approval Workflow"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Status
            parent.spawn((
                Text::new("Status: Ready"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                StatusText,
            ));

            // Current node description
            parent.spawn((
                Text::new("Press SPACE to start the workflow animation"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                CurrentNodeText,
            ));
        });

        // Side panel for node list
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(120.0),
                right: Val::Px(10.0),
                width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Workflow Steps:"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                NodeListText,
            ));
        });

        // Controls help
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Controls: SPACE=Start/Pause | R=Reset | Mouse=Camera | ESC=Exit"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
    });
}

fn update_ui(
    workflow: Res<WorkflowState>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<NodeListText>, Without<CurrentNodeText>)>,
    mut node_list_query: Query<&mut Text, (With<NodeListText>, Without<StatusText>, Without<CurrentNodeText>)>,
    mut current_node_query: Query<&mut Text, (With<CurrentNodeText>, Without<StatusText>, Without<NodeListText>)>,
) {
    // Update status text
    if let Ok(mut text) = status_query.get_single_mut() {
        text.0 = format!("Status: {}", 
            if workflow.is_running { "▶️ Running" } else { "⏸️ Paused" }
        );
    }

    // Update current node description
    if let Ok(mut text) = current_node_query.get_single_mut() {
        if workflow.current_step < workflow.nodes.len() {
            let node = &workflow.nodes[workflow.current_step];
            text.0 = format!("Current: {} - {}", node.name, node.description);
        }
    }

    // Update node list
    if let Ok(mut text) = node_list_query.get_single_mut() {
        let mut list = String::new();
        for (i, node) in workflow.nodes.iter().enumerate() {
            let status_icon = match node.status {
                NodeStatus::Completed => "✅",
                NodeStatus::Active => "▶️",
                NodeStatus::Pending => "⏳",
            };
            
            let highlight = if i == workflow.current_step { ">>> " } else { "    " };
            list.push_str(&format!("{}{} {} {}\n", highlight, status_icon, (i + 1), node.name));
        }
        text.0 = list;
    }
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
    
    let edge_mesh = meshes.add(Cylinder::new(0.08, length));
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.5),
        emissive: LinearRgba::from(Color::srgb(0.1, 0.1, 0.15)),
        metallic: 0.3,
        perceptual_roughness: 0.7,
        ..default()
    });
    
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

fn camera_controls(
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    for event in mouse_wheel.read() {
        camera_state.distance -= event.y * 2.0;
        camera_state.distance = camera_state.distance.clamp(5.0, 50.0);
    }

    if mouse_input.pressed(MouseButton::Right) {
        for event in mouse_motion.read() {
            camera_state.rotation -= event.delta.x * 0.01;
            camera_state.height += event.delta.y * 0.1;
            camera_state.height = camera_state.height.clamp(-10.0, 30.0);
        }
    }

    let move_speed = 10.0 * time.delta_secs();
    if keyboard.pressed(KeyCode::ArrowLeft) {
        camera_state.rotation += move_speed * 0.5;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        camera_state.rotation -= move_speed * 0.5;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        camera_state.distance -= move_speed * 5.0;
        camera_state.distance = camera_state.distance.max(5.0);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        camera_state.distance += move_speed * 5.0;
        camera_state.distance = camera_state.distance.min(50.0);
    }

    let x = camera_state.rotation.cos() * camera_state.distance;
    let z = camera_state.rotation.sin() * camera_state.distance;
    camera_transform.translation = Vec3::new(x, camera_state.height, z);
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn rotate_nodes(
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<WorkflowNode>, Without<MainCamera>)>,
) {
    for mut transform in query.iter_mut() {
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
        // Update node statuses
        let current = workflow.current_step;
        if current < workflow.nodes.len() {
            workflow.nodes[current].status = NodeStatus::Completed;
        }

        // Reset previous node color
        for (node, material_handle) in nodes.iter() {
            if node.index == workflow.current_step {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = LinearRgba::from(material.base_color) * 0.02;
                }
            }
        }

        // Move to next step
        workflow.current_step = (workflow.current_step + 1) % 7;

        // Update new node status
        let new_current = workflow.current_step;
        if new_current < workflow.nodes.len() {
            workflow.nodes[new_current].status = NodeStatus::Active;
        }

        // Highlight current node
        for (node, material_handle) in nodes.iter() {
            if node.index == workflow.current_step {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = LinearRgba::from(material.base_color) * 0.5;
                }
            }
        }

        // Reset if completed
        if workflow.current_step == 0 {
            for node in workflow.nodes.iter_mut() {
                node.status = NodeStatus::Pending;
            }
            workflow.nodes[0].status = NodeStatus::Active;
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workflow: ResMut<WorkflowState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        workflow.is_running = !workflow.is_running;
        if workflow.is_running {
            let current = workflow.current_step;
            if current < workflow.nodes.len() {
                workflow.nodes[current].status = NodeStatus::Active;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        workflow.is_running = false;
        workflow.current_step = 0;
        workflow.timer.reset();

        for node in workflow.nodes.iter_mut() {
            node.status = NodeStatus::Pending;
        }

        for (_, material_handle) in nodes.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.emissive = LinearRgba::from(material.base_color) * 0.02;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

#[derive(Clone, Copy)]
enum NodeType {
    Start,
    Task,
    Decision,
    End,
} 