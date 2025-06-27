//! Bevy Domain Projections
//!
//! In ECS, projections are resources that maintain derived state.
//! These act as read models that are updated by systems processing events.

use crate::events::*;
use crate::value_objects::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Graph view projection - maintains graph structure for queries
#[derive(Resource, Default)]
pub struct GraphViewProjection {
    pub nodes: HashMap<NodeId, NodeView>,
    pub edges: HashMap<EdgeId, EdgeView>,
    pub node_edges: HashMap<NodeId, HashSet<EdgeId>>,
    pub selected_nodes: HashSet<NodeId>,
}

/// View of a node in the projection
#[derive(Clone, Debug)]
pub struct NodeView {
    pub entity: Entity,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub is_selected: bool,
}

/// View of an edge in the projection
#[derive(Clone, Debug)]
pub struct EdgeView {
    pub entity: Entity,
    pub source_node_id: NodeId,
    pub target_node_id: NodeId,
}

/// System that updates the graph projection from events
pub fn update_graph_projection(
    mut projection: ResMut<GraphViewProjection>,
    mut node_created: EventReader<VisualNodeCreated>,
    mut edge_created: EventReader<VisualEdgeCreated>,
    mut node_moved: EventReader<NodeMoved>,
    mut node_selected: EventReader<NodeSelected>,
    mut node_deselected: EventReader<NodeDeselected>,
    mut node_deleted: EventReader<VisualNodeDeleted>,
    mut edge_deleted: EventReader<VisualEdgeDeleted>,
) {
    // Handle node creation
    for event in node_created.read() {
        let view = NodeView {
            entity: event.entity,
            position: event.position.clone(),
            metadata: NodeMetadata::default(),
            is_selected: false,
        };
        projection.nodes.insert(event.node_id.clone(), view);
    }

    // Handle edge creation
    for event in edge_created.read() {
        let view = EdgeView {
            entity: event.entity,
            source_node_id: NodeId(Uuid::new_v4()), // Would need to look up from entity
            target_node_id: NodeId(Uuid::new_v4()), // Would need to look up from entity
        };
        projection.edges.insert(event.edge_id.clone(), view);
    }

    // Handle node movement
    for event in node_moved.read() {
        if let Some(node) = projection.nodes.get_mut(&event.node_id) {
            node.position = event.new_position.clone();
        }
    }

    // Handle selection
    for event in node_selected.read() {
        projection.selected_nodes.insert(event.node_id.clone());
        if let Some(node) = projection.nodes.get_mut(&event.node_id) {
            node.is_selected = true;
        }
    }

    // Handle deselection
    for event in node_deselected.read() {
        projection.selected_nodes.remove(&event.node_id);
        if let Some(node) = projection.nodes.get_mut(&event.node_id) {
            node.is_selected = false;
        }
    }

    // Handle node deletion
    for event in node_deleted.read() {
        projection.nodes.remove(&event.node_id);
        projection.selected_nodes.remove(&event.node_id);
        projection.node_edges.remove(&event.node_id);
    }

    // Handle edge deletion
    for event in edge_deleted.read() {
        projection.edges.remove(&event.edge_id);
    }
}

/// Spatial index projection for efficient spatial queries
#[derive(Resource, Default)]
pub struct SpatialIndexProjection {
    // In a real implementation, this would use an R-tree or similar
    pub node_positions: Vec<(NodeId, Position)>,
}

/// System that updates spatial index
pub fn update_spatial_index(
    mut index: ResMut<SpatialIndexProjection>,
    mut node_created: EventReader<VisualNodeCreated>,
    mut node_moved: EventReader<NodeMoved>,
    mut node_deleted: EventReader<VisualNodeDeleted>,
) {
    for event in node_created.read() {
        index
            .node_positions
            .push((event.node_id.clone(), event.position.clone()));
    }

    for event in node_moved.read() {
        if let Some(entry) = index
            .node_positions
            .iter_mut()
            .find(|(id, _)| id == &event.node_id)
        {
            entry.1 = event.new_position.clone();
        }
    }

    for event in node_deleted.read() {
        index.node_positions.retain(|(id, _)| id != &event.node_id);
    }
}

/// Plugin that registers projection systems
pub struct ProjectionPlugin;

impl Plugin for ProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphViewProjection>()
            .init_resource::<SpatialIndexProjection>()
            .add_systems(Update, (update_graph_projection, update_spatial_index));
    }
}
