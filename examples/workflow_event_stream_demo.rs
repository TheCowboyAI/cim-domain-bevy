//! Workflow Event Stream Demo
//!
//! Shows the actual domain events being generated as the workflow progresses.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use uuid::Uuid;

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
        .insert_resource(EventStream::default())
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

// Domain Events
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WorkflowEvent {
    WorkflowStarted {
        workflow_id: Uuid,
        timestamp: String,
        initiator: String,
    },
    DocumentSubmitted {
        document_id: Uuid,
        workflow_id: Uuid,
        title: String,
        size_bytes: u64,
        timestamp: String,
    },
    ReviewStarted {
        document_id: Uuid,
        reviewer: String,
        review_type: String,
        timestamp: String,
    },
    DecisionRequested {
        document_id: Uuid,
        decision_engine: String,
        rules_count: u32,
        timestamp: String,
    },
    DocumentApproved {
        document_id: Uuid,
        approver: String,
        confidence_score: f32,
        timestamp: String,
    },
    DocumentRejected {
        document_id: Uuid,
        reason: String,
        can_revise: bool,
        timestamp: String,
    },
    RevisionRequested {
        document_id: Uuid,
        required_changes: Vec<String>,
        deadline: String,
        timestamp: String,
    },
}

#[derive(Resource)]
struct EventStream {
    events: VecDeque<(WorkflowEvent, f32)>, // Event and time since added
    max_events: usize,
}

impl Default for EventStream {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            max_events: 20,
        }
    }
}

impl EventStream {
    fn add_event(&mut self, event: WorkflowEvent) {
        self.events.push_front((event, 0.0));
        if self.events.len() > self.max_events {
            self.events.pop_back();
        }
    }

    fn update(&mut self, delta: f32) {
        for (_, age) in self.events.iter_mut() {
            *age += delta;
        }
    }
}

#[derive(Resource)]
struct WorkflowState {
    current_step: usize,
    timer: Timer,
    is_running: bool,
    workflow_id: Uuid,
    document_id: Uuid,
    nodes: Vec<NodeInfo>,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            current_step: 0,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            is_running: false,
            workflow_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            nodes: vec![
                NodeInfo {
                    name: "Start".to_string(),
                    description: "Workflow initialization".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Submit".to_string(),
                    description: "Document submission and validation".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Review".to_string(),
                    description: "Automated compliance review".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Decision".to_string(),
                    description: "AI-powered decision making".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Approved".to_string(),
                    description: "Document approved and archived".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Rejected".to_string(),
                    description: "Document rejected with reasons".to_string(),
                    status: NodeStatus::Pending,
                },
                NodeInfo {
                    name: "Revise".to_string(),
                    description: "Revision requested with feedback".to_string(),
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
            distance: 30.0,
            rotation: 0.0,
            height: 12.0,
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
struct EventListText;

#[derive(Component)]
struct CurrentStepText;

#[derive(Component)]
struct EventCountText;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Workflow nodes
    let nodes = vec![
        ("Start", Vec3::new(-15.0, 0.0, 0.0), NodeType::Start),
        ("Submit", Vec3::new(-8.0, 0.0, 0.0), NodeType::Task),
        ("Review", Vec3::new(-1.0, 0.0, 0.0), NodeType::Task),
        ("Decision", Vec3::new(6.0, 0.0, 0.0), NodeType::Decision),
        ("Approved", Vec3::new(13.0, 0.0, 4.0), NodeType::End),
        ("Rejected", Vec3::new(13.0, 0.0, -4.0), NodeType::End),
        ("Revise", Vec3::new(-1.0, 0.0, -8.0), NodeType::Task),
    ];

    // Create nodes
    for (i, (name, position, node_type)) in nodes.iter().enumerate() {
        let (mesh, color) = match node_type {
            NodeType::Start => (
                meshes.add(Sphere::new(1.5)),
                Color::srgb(0.2, 0.8, 0.2),
            ),
            NodeType::Task => (
                meshes.add(Cylinder::new(1.2, 2.0)),
                Color::srgb(0.3, 0.5, 0.8),
            ),
            NodeType::Decision => (
                meshes.add(Cuboid::new(2.5, 2.5, 2.5)),
                Color::srgb(0.8, 0.8, 0.2),
            ),
            NodeType::End => (
                meshes.add(Sphere::new(1.5)),
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
        (nodes[0].1, nodes[1].1),
        (nodes[1].1, nodes[2].1),
        (nodes[2].1, nodes[3].1),
        (nodes[3].1, nodes[4].1),
        (nodes[3].1, nodes[5].1),
        (nodes[3].1, nodes[6].1),
        (nodes[6].1, nodes[2].1),
    ];

    for (start, end) in edges {
        create_edge(&mut commands, &mut meshes, &mut materials, start, end);
    }
}

fn setup_ui(mut commands: Commands) {
    // Root container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::NONE.into()),
    )).with_children(|parent| {
        // Event stream panel (left side)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                width: Val::Px(500.0),
                max_height: Val::Percent(80.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("🔄 Event Stream"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Event count
            parent.spawn((
                Text::new("Events: 0"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                EventCountText,
            ));

            // Separator
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1).into()),
            ));

            // Event list
            parent.spawn((
                Text::new("Waiting to start..."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                EventListText,
            ));
        });

        // Current step info (top right)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(350.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("📊 Document Approval Workflow"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("\nPress SPACE to start"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                CurrentStepText,
            ));
        });

        // Controls (bottom)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9).into()),
            BorderRadius::all(Val::Px(10.0)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Controls: SPACE=Start/Pause | R=Reset | Mouse=Camera | ESC=Exit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
    });
}

fn update_ui(
    workflow: Res<WorkflowState>,
    event_stream: Res<EventStream>,
    mut event_list_query: Query<&mut Text, (With<EventListText>, Without<CurrentStepText>, Without<EventCountText>)>,
    mut current_step_query: Query<&mut Text, (With<CurrentStepText>, Without<EventListText>, Without<EventCountText>)>,
    mut event_count_query: Query<&mut Text, (With<EventCountText>, Without<EventListText>, Without<CurrentStepText>)>,
) {
    // Update event count
    if let Ok(mut text) = event_count_query.get_single_mut() {
        text.0 = format!("Events: {} | Status: {}", 
            event_stream.events.len(),
            if workflow.is_running { "▶️ Running" } else { "⏸️ Paused" }
        );
    }

    // Update current step
    if let Ok(mut text) = current_step_query.get_single_mut() {
        if workflow.current_step < workflow.nodes.len() {
            let node = &workflow.nodes[workflow.current_step];
            text.0 = format!("\nCurrent Step: {}\n{}\n\nWorkflow ID: {:?}\nDocument ID: {:?}", 
                node.name, 
                node.description,
                workflow.workflow_id.to_string().split('-').next().unwrap_or(""),
                workflow.document_id.to_string().split('-').next().unwrap_or("")
            );
        }
    }

    // Update event list
    if let Ok(mut text) = event_list_query.get_single_mut() {
        let mut event_text = String::new();
        
        for (event, age) in event_stream.events.iter() {
            let opacity = (1.0 - (age / 10.0)).max(0.3);
            let formatted_event = format_event(event);
            
            // Add event with age indicator
            if *age < 0.5 {
                event_text.push_str("🆕 ");
            } else if *age < 2.0 {
                event_text.push_str("   ");
            } else {
                event_text.push_str("   ");
            }
            
            event_text.push_str(&formatted_event);
            event_text.push_str("\n\n");
        }
        
        if event_text.is_empty() {
            event_text = "Waiting for events...".to_string();
        }
        
        text.0 = event_text;
    }
}

fn format_event(event: &WorkflowEvent) -> String {
    match event {
        WorkflowEvent::WorkflowStarted { workflow_id, timestamp, initiator } => {
            format!("🚀 WorkflowStarted\n   ID: {}\n   Initiator: {}\n   Time: {}", 
                workflow_id.to_string().split('-').next().unwrap_or(""),
                initiator,
                timestamp
            )
        }
        WorkflowEvent::DocumentSubmitted { document_id, title, size_bytes, timestamp, .. } => {
            format!("📄 DocumentSubmitted\n   ID: {}\n   Title: \"{}\"\n   Size: {} KB\n   Time: {}", 
                document_id.to_string().split('-').next().unwrap_or(""),
                title,
                size_bytes / 1024,
                timestamp
            )
        }
        WorkflowEvent::ReviewStarted { reviewer, review_type, timestamp, .. } => {
            format!("🔍 ReviewStarted\n   Reviewer: {}\n   Type: {}\n   Time: {}", 
                reviewer,
                review_type,
                timestamp
            )
        }
        WorkflowEvent::DecisionRequested { decision_engine, rules_count, timestamp, .. } => {
            format!("🤖 DecisionRequested\n   Engine: {}\n   Rules: {} rules\n   Time: {}", 
                decision_engine,
                rules_count,
                timestamp
            )
        }
        WorkflowEvent::DocumentApproved { approver, confidence_score, timestamp, .. } => {
            format!("✅ DocumentApproved\n   Approver: {}\n   Confidence: {:.1}%\n   Time: {}", 
                approver,
                confidence_score * 100.0,
                timestamp
            )
        }
        WorkflowEvent::DocumentRejected { reason, can_revise, timestamp, .. } => {
            format!("❌ DocumentRejected\n   Reason: {}\n   Can Revise: {}\n   Time: {}", 
                reason,
                if *can_revise { "Yes" } else { "No" },
                timestamp
            )
        }
        WorkflowEvent::RevisionRequested { required_changes, deadline, timestamp, .. } => {
            format!("📝 RevisionRequested\n   Changes: {} required\n   Deadline: {}\n   Time: {}", 
                required_changes.len(),
                deadline,
                timestamp
            )
        }
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
    
    let edge_mesh = meshes.add(Cylinder::new(0.1, length));
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.5),
        emissive: LinearRgba::from(Color::srgb(0.1, 0.1, 0.15)),
        metallic: 0.3,
        perceptual_roughness: 0.7,
        ..default()
    });
    
    let up = if direction.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
    
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
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    for event in mouse_wheel.read() {
        camera_state.distance -= event.y * 2.0;
        camera_state.distance = camera_state.distance.clamp(10.0, 60.0);
    }

    if mouse_input.pressed(MouseButton::Right) {
        for event in mouse_motion.read() {
            camera_state.rotation -= event.delta.x * 0.01;
            camera_state.height += event.delta.y * 0.1;
            camera_state.height = camera_state.height.clamp(-10.0, 30.0);
        }
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
    mut event_stream: ResMut<EventStream>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&WorkflowNode, &MeshMaterial3d<StandardMaterial>)>,
) {
    // Update event ages
    event_stream.update(time.delta_secs());

    if !workflow.is_running {
        return;
    }

    workflow.timer.tick(time.delta());

    if workflow.timer.just_finished() {
        // Generate event for current step
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        
        let event = match workflow.current_step {
            0 => WorkflowEvent::WorkflowStarted {
                workflow_id: workflow.workflow_id,
                timestamp,
                initiator: "John Smith".to_string(),
            },
            1 => WorkflowEvent::DocumentSubmitted {
                document_id: workflow.document_id,
                workflow_id: workflow.workflow_id,
                title: "Q4 Financial Report.pdf".to_string(),
                size_bytes: 2_456_789,
                timestamp,
            },
            2 => WorkflowEvent::ReviewStarted {
                document_id: workflow.document_id,
                reviewer: "ComplianceBot v2.1".to_string(),
                review_type: "Automated Compliance Check".to_string(),
                timestamp,
            },
            3 => WorkflowEvent::DecisionRequested {
                document_id: workflow.document_id,
                decision_engine: "CIM Decision Engine".to_string(),
                rules_count: 47,
                timestamp,
            },
            4 => WorkflowEvent::DocumentApproved {
                document_id: workflow.document_id,
                approver: "AI Approval System".to_string(),
                confidence_score: 0.943,
                timestamp,
            },
            5 => WorkflowEvent::DocumentRejected {
                document_id: workflow.document_id,
                reason: "Missing required signatures".to_string(),
                can_revise: true,
                timestamp,
            },
            6 => WorkflowEvent::RevisionRequested {
                document_id: workflow.document_id,
                required_changes: vec![
                    "Add CFO signature".to_string(),
                    "Update risk assessment section".to_string(),
                    "Include Q3 comparison data".to_string(),
                ],
                deadline: "2024-01-15".to_string(),
                timestamp,
            },
            _ => WorkflowEvent::WorkflowStarted {
                workflow_id: workflow.workflow_id,
                timestamp,
                initiator: "System".to_string(),
            },
        };

        event_stream.add_event(event);

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

        // Move to next step (follow specific path)
        workflow.current_step = match workflow.current_step {
            0 => 1, // Start -> Submit
            1 => 2, // Submit -> Review
            2 => 3, // Review -> Decision
            3 => 4, // Decision -> Approved (for demo)
            4 => 0, // Approved -> Start (restart)
            _ => 0,
        };

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
            workflow.workflow_id = Uuid::new_v4();
            workflow.document_id = Uuid::new_v4();
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
    mut event_stream: ResMut<EventStream>,
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
        workflow.workflow_id = Uuid::new_v4();
        workflow.document_id = Uuid::new_v4();
        event_stream.events.clear();

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