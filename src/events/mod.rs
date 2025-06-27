//! Bevy Domain Events
//!
//! Domain events represent things that have happened in the visual domain.
//! These are emitted by systems after processing commands.

use crate::value_objects::*;
use bevy::prelude::*;

/// Event: A visual node was created
#[derive(Event, Debug, Clone)]
pub struct VisualNodeCreated {
    pub entity: Entity,
    pub node_id: NodeId,
    pub position: Position,
}

/// Event: A visual edge was created
#[derive(Event, Debug, Clone)]
pub struct VisualEdgeCreated {
    pub entity: Entity,
    pub edge_id: EdgeId,
    pub source_entity: Entity,
    pub target_entity: Entity,
}

/// Event: A node was moved
#[derive(Event, Debug, Clone)]
pub struct NodeMoved {
    pub entity: Entity,
    pub node_id: NodeId,
    pub old_position: Position,
    pub new_position: Position,
}

/// Event: A node's style was updated
#[derive(Event, Debug, Clone)]
pub struct NodeStyleUpdated {
    pub entity: Entity,
    pub node_id: NodeId,
    pub old_style: NodeVisualStyle,
    pub new_style: NodeVisualStyle,
}

/// Event: A visual node was deleted
#[derive(Event, Debug, Clone)]
pub struct VisualNodeDeleted {
    pub node_id: NodeId,
    pub final_position: Position,
}

/// Event: A visual edge was deleted
#[derive(Event, Debug, Clone)]
pub struct VisualEdgeDeleted {
    pub edge_id: EdgeId,
}

/// Event: A node was selected
#[derive(Event, Debug, Clone)]
pub struct NodeSelected {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: A node was deselected
#[derive(Event, Debug, Clone)]
pub struct NodeDeselected {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: Canvas was panned
#[derive(Event, Debug, Clone)]
pub struct CanvasPanned {
    pub delta: Vec2,
    pub new_offset: Vec2,
}

/// Event: Canvas was zoomed
#[derive(Event, Debug, Clone)]
pub struct CanvasZoomed {
    pub old_zoom: f32,
    pub new_zoom: f32,
    pub focal_point: Vec2,
}
