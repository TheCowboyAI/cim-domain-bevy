//! Bevy Domain Commands
//!
//! Commands in ECS are events that trigger systems to perform operations.
//! Systems act as command handlers that process these commands.

use crate::value_objects::*;
use bevy::prelude::*;

/// Command to create a visual node
#[derive(Event, Debug, Clone)]
pub struct CreateVisualNode {
    pub node_id: NodeId,
    pub position: Position,
    pub visual_style: NodeVisualStyle,
}

/// Command to create a visual edge
#[derive(Event, Debug, Clone)]
pub struct CreateVisualEdge {
    pub edge_id: EdgeId,
    pub source_node_id: NodeId,
    pub target_node_id: NodeId,
    pub edge_style: EdgeVisualStyle,
}

/// Command to move a node
#[derive(Event, Debug, Clone)]
pub struct MoveNode {
    pub node_id: NodeId,
    pub new_position: Position,
    pub animate: bool,
}

/// Command to update node visual style
#[derive(Event, Debug, Clone)]
pub struct UpdateNodeStyle {
    pub node_id: NodeId,
    pub new_style: NodeVisualStyle,
}

/// Command to delete a visual node
#[derive(Event, Debug, Clone)]
pub struct DeleteVisualNode {
    pub node_id: NodeId,
}

/// Command to delete a visual edge
#[derive(Event, Debug, Clone)]
pub struct DeleteVisualEdge {
    pub edge_id: EdgeId,
}

/// Command to select a node
#[derive(Event, Debug, Clone)]
pub struct SelectNode {
    pub node_id: NodeId,
    pub multi_select: bool,
}

/// Command to pan the canvas
#[derive(Event, Debug, Clone)]
pub struct PanCanvas {
    pub delta: Vec2,
}

/// Command to zoom the canvas
#[derive(Event, Debug, Clone)]
pub struct ZoomCanvas {
    pub zoom_factor: f32,
    pub focal_point: Option<Vec2>,
}
