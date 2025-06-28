//! Bevy Domain Events
//!
//! Domain events represent things that have happened in the visual domain.
//! These are emitted by systems after processing commands.

use bevy::prelude::*;
use uuid::Uuid;
use cim_contextgraph::{NodeId, EdgeId};

/// Position type for events
pub type Position = Vec3;

/// Node visual style
#[derive(Debug, Clone, PartialEq)]
pub struct NodeVisualStyle {
    pub color: Color,
    pub size: f32,
    pub shape: EventNodeShape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventNodeShape {
    Circle,
    Square,
    Diamond,
}

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

// Visualization Commands (these are like commands but implemented as events for simplicity)

/// Command to create a node visual
#[derive(Event, Debug, Clone)]
pub struct CreateNodeVisual {
    pub node_id: Uuid,
    pub position: Vec3,
    pub label: String,
}

/// Command to remove a node visual
#[derive(Event, Debug, Clone)]
pub struct RemoveNodeVisual {
    pub node_id: Uuid,
}

/// Command to create an edge visual
#[derive(Event, Debug, Clone)]
pub struct CreateEdgeVisual {
    pub edge_id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub relationship: EdgeRelationship,
}

/// Command to remove an edge visual
#[derive(Event, Debug, Clone)]
pub struct RemoveEdgeVisual {
    pub edge_id: Uuid,
}

/// Edge relationship types
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeRelationship {
    DependsOn,
    Contains,
    References,
    Custom(String),
}

/// Visualization command type
#[derive(Event, Debug, Clone)]
pub enum VisualizationCommand {
    CreateNode(CreateNodeVisual),
    RemoveNode(RemoveNodeVisual),
    CreateEdge(CreateEdgeVisual),
    RemoveEdge(RemoveEdgeVisual),
}

// Interaction Events

/// Event: Node was clicked
#[derive(Event, Debug, Clone)]
pub struct NodeClicked {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: Node was hovered
#[derive(Event, Debug, Clone)]
pub struct NodeHovered {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: Node was unhovered
#[derive(Event, Debug, Clone)]
pub struct NodeUnhovered {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: Edge was clicked
#[derive(Event, Debug, Clone)]
pub struct EdgeClicked {
    pub entity: Entity,
    pub edge_id: EdgeId,
}

/// Event: Node drag started
#[derive(Event, Debug, Clone)]
pub struct NodeDragStart {
    pub entity: Entity,
    pub node_id: NodeId,
    pub start_position: Vec3,
}

/// Event: Node is being dragged
#[derive(Event, Debug, Clone)]
pub struct NodeDragging {
    pub entity: Entity,
    pub node_id: NodeId,
    pub current_position: Vec3,
}

/// Event: Node drag ended
#[derive(Event, Debug, Clone)]
pub struct NodeDragEnd {
    pub entity: Entity,
    pub node_id: NodeId,
    pub final_position: Vec3,
}

/// Event: Node position changed
#[derive(Event, Debug, Clone)]
pub struct NodePositionChanged {
    pub entity: Entity,
    pub node_id: NodeId,
    pub old_position: Vec3,
    pub new_position: Vec3,
}

/// Event: Node metadata changed
#[derive(Event, Debug, Clone)]
pub struct NodeMetadataChanged {
    pub entity: Entity,
    pub node_id: NodeId,
}

/// Event: Selection changed
#[derive(Event, Debug, Clone)]
pub struct SelectionChanged {
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
}
