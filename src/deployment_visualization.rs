//! Deployment graph visualization components and systems

use bevy::prelude::*;
use cim_domain_graph::deployment::{DeploymentNodeType, DeploymentEdgeType};
use cim_contextgraph::NodeId;
use serde::{Deserialize, Serialize};

/// Component marking deployment-specific nodes
#[derive(Component, Debug, Clone)]
pub struct DeploymentNode {
    pub node_type: DeploymentNodeType,
    pub status: DeploymentStatus,
}

/// Deployment status for visual representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Not yet deployed
    Pending,
    /// Currently deploying
    Deploying,
    /// Successfully deployed and running
    Running,
    /// Deployment failed
    Failed,
    /// Service is stopped
    Stopped,
}

impl DeploymentStatus {
    /// Get the color for this status
    pub fn color(&self) -> Color {
        match self {
            Self::Pending => Color::srgb(0.5, 0.5, 0.5),
            Self::Deploying => Color::srgb(1.0, 0.8, 0.0),
            Self::Running => Color::srgb(0.0, 0.8, 0.0),
            Self::Failed => Color::srgb(0.8, 0.0, 0.0),
            Self::Stopped => Color::srgb(0.3, 0.3, 0.3),
        }
    }
}

/// Component for deployment edge visualization
#[derive(Component, Debug, Clone)]
pub struct DeploymentEdge {
    pub edge_type: DeploymentEdgeType,
    pub from_node: NodeId,
    pub to_node: NodeId,
}

/// Resource for deployment visualization settings
#[derive(Resource, Debug, Clone)]
pub struct DeploymentVisualizationSettings {
    pub show_resource_usage: bool,
    pub show_dependencies: bool,
    pub show_network_connections: bool,
    pub animate_data_flow: bool,
    pub highlight_critical_path: bool,
}

impl Default for DeploymentVisualizationSettings {
    fn default() -> Self {
        Self {
            show_resource_usage: true,
            show_dependencies: true,
            show_network_connections: true,
            animate_data_flow: false,
            highlight_critical_path: false,
        }
    }
}

/// Plugin for deployment visualization
pub struct DeploymentVisualizationPlugin;

impl Plugin for DeploymentVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DeploymentVisualizationSettings::default())
            .add_systems(Update, (
                update_deployment_node_colors,
                animate_deploying_nodes,
                highlight_dependencies,
                show_resource_overlays,
            ));
    }
}

/// Update node colors based on deployment status
fn update_deployment_node_colors(
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&DeploymentNode, &MeshMaterial3d<StandardMaterial>), Changed<DeploymentNode>>,
) {
    for (deployment_node, material_handle) in nodes.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = deployment_node.status.color();
            
            // Add emissive glow for deploying nodes
            if deployment_node.status == DeploymentStatus::Deploying {
                material.emissive = LinearRgba::from(deployment_node.status.color()) * 0.5;
            } else {
                material.emissive = LinearRgba::BLACK;
            }
        }
    }
}

/// Animate nodes that are currently deploying
fn animate_deploying_nodes(
    time: Res<Time>,
    mut nodes: Query<(&DeploymentNode, &mut Transform), With<DeploymentNode>>,
) {
    for (deployment_node, mut transform) in nodes.iter_mut() {
        if deployment_node.status == DeploymentStatus::Deploying {
            // Pulse effect
            let pulse = (time.elapsed_secs() * 2.0).sin() * 0.1 + 1.0;
            transform.scale = Vec3::splat(pulse);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}

/// Highlight dependency relationships
fn highlight_dependencies(
    settings: Res<DeploymentVisualizationSettings>,
    deployment_edges: Query<&DeploymentEdge>,
    mut gizmos: Gizmos,
    transforms: Query<&Transform>,
    node_entities: Query<(Entity, &NodeId), With<DeploymentNode>>,
) {
    if !settings.show_dependencies {
        return;
    }
    
    // This would need proper entity mapping in a real implementation
    // For now, it's a placeholder showing the structure
}

/// Show resource usage overlays
fn show_resource_overlays(
    settings: Res<DeploymentVisualizationSettings>,
    deployment_nodes: Query<(&DeploymentNode, &Transform)>,
    mut gizmos: Gizmos,
) {
    if !settings.show_resource_usage {
        return;
    }
    
    for (deployment_node, transform) in deployment_nodes.iter() {
        match &deployment_node.node_type {
            DeploymentNodeType::Service { resources, .. } |
            DeploymentNodeType::Database { resources, .. } |
            DeploymentNodeType::Agent { resources, .. } => {
                // Draw resource bars above the node
                let base_pos = transform.translation + Vec3::Y * 3.0;
                
                // CPU usage bar
                if let Some(cpu) = resources.cpu_cores {
                    let cpu_width = cpu.min(4.0); // Cap display at 4 cores
                    gizmos.rect(
                        base_pos + Vec3::Y * 0.0,
                        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        Vec2::new(cpu_width, 0.2),
                        Color::srgb(0.2, 0.8, 0.2),
                    );
                }
                
                // Memory usage bar
                if let Some(memory_mb) = resources.memory_mb {
                    let memory_gb = memory_mb as f32 / 1024.0;
                    let memory_width = memory_gb.min(8.0) / 2.0; // Scale to reasonable size
                    gizmos.rect(
                        base_pos + Vec3::Y * 0.3,
                        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        Vec2::new(memory_width, 0.2),
                        Color::srgb(0.2, 0.2, 0.8),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Bundle for creating a deployment node visual
#[derive(Bundle)]
pub struct DeploymentNodeBundle {
    pub deployment: DeploymentNode,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl DeploymentNodeBundle {
    /// Create a new deployment node bundle
    pub fn new(
        node_type: DeploymentNodeType,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
    ) -> Self {
        Self {
            deployment: DeploymentNode {
                node_type,
                status: DeploymentStatus::Pending,
            },
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}