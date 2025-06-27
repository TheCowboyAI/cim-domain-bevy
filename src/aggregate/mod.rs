//! Bevy Domain Aggregates
//!
//! In ECS, aggregates are entities with specific component compositions
//! that enforce business rules and invariants.

use crate::events::*;
use crate::value_objects::*;
use bevy::prelude::*;

/// Visual Node Aggregate - An entity that represents a visual node
/// Components define the state, systems enforce the rules
#[derive(Bundle)]
pub struct VisualNodeAggregate {
    pub node_id: NodeId,
    pub position: Position,
    pub visual: NodeVisual,
    pub interaction_state: InteractionState,
    pub metadata: NodeMetadata,
}

/// Visual Edge Aggregate - An entity that represents a visual edge
#[derive(Bundle)]
pub struct VisualEdgeAggregate {
    pub edge_id: EdgeId,
    pub source: SourceNode,
    pub target: TargetNode,
    pub visual: EdgeVisual,
    pub curve: EdgeCurve,
}

/// Graph Canvas Aggregate - The root aggregate for visual graph
#[derive(Bundle)]
pub struct GraphCanvasAggregate {
    pub graph_id: GraphId,
    pub canvas_state: CanvasState,
    pub viewport: Viewport,
    pub render_settings: RenderSettings,
}

impl VisualNodeAggregate {
    pub fn new(node_id: NodeId, position: Position) -> Self {
        Self {
            node_id,
            position,
            visual: NodeVisual::default(),
            interaction_state: InteractionState::default(),
            metadata: NodeMetadata::default(),
        }
    }
}

impl VisualEdgeAggregate {
    pub fn new(edge_id: EdgeId, source: Entity, target: Entity) -> Self {
        Self {
            edge_id,
            source: SourceNode(source),
            target: TargetNode(target),
            visual: EdgeVisual::default(),
            curve: EdgeCurve::default(),
        }
    }
}

impl GraphCanvasAggregate {
    pub fn new(graph_id: GraphId) -> Self {
        Self {
            graph_id,
            canvas_state: CanvasState::default(),
            viewport: Viewport::default(),
            render_settings: RenderSettings::default(),
        }
    }
}
