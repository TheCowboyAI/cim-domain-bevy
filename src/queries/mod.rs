//! Bevy Domain Queries
//!
//! In ECS, queries are systems that read component data to answer questions.
//! These are wrapped to provide a domain-oriented interface.

use crate::value_objects::*;
use bevy::prelude::*;

/// Query to find all nodes in a graph
pub fn query_nodes_in_graph(
    graph_id: GraphId,
    nodes: Query<(Entity, &NodeId, &Position), With<NodeId>>,
) -> Vec<(Entity, NodeId, Position)> {
    nodes
        .iter()
        .map(|(e, id, pos)| (e, id.clone(), pos.clone()))
        .collect()
}

/// Query to find selected nodes
pub fn query_selected_nodes(
    nodes: Query<(Entity, &NodeId, &InteractionState)>,
) -> Vec<(Entity, NodeId)> {
    nodes
        .iter()
        .filter(|(_, _, state)| state.is_selected)
        .map(|(e, id, _)| (e, id.clone()))
        .collect()
}

/// Query to find nodes within a region
pub fn query_nodes_in_region(
    min: Vec2,
    max: Vec2,
    nodes: Query<(Entity, &NodeId, &Position)>,
) -> Vec<(Entity, NodeId)> {
    nodes
        .iter()
        .filter(|(_, _, pos)| pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y)
        .map(|(e, id, _)| (e, id.clone()))
        .collect()
}

/// Query to find edges connected to a node
pub fn query_edges_for_node(
    node_entity: Entity,
    edges: Query<(Entity, &EdgeId, &SourceNode, &TargetNode)>,
) -> Vec<(Entity, EdgeId)> {
    edges
        .iter()
        .filter(|(_, _, source, target)| source.0 == node_entity || target.0 == node_entity)
        .map(|(e, id, _, _)| (e, id.clone()))
        .collect()
}

/// Query system that finds the nearest node to a position
pub fn find_nearest_node_system(
    position: Vec2,
    max_distance: f32,
    nodes: Query<(Entity, &NodeId, &Position)>,
) -> Option<(Entity, NodeId, f32)> {
    let mut nearest = None;
    let mut min_distance = max_distance;

    for (entity, node_id, node_pos) in nodes.iter() {
        let distance = Vec2::new(node_pos.x, node_pos.y).distance(position);
        if distance < min_distance {
            min_distance = distance;
            nearest = Some((entity, node_id.clone(), distance));
        }
    }

    nearest
}

/// Query to get graph statistics
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub selected_count: usize,
}

pub fn query_graph_statistics(
    nodes: Query<&NodeId>,
    edges: Query<&EdgeId>,
    selected: Query<&InteractionState>,
) -> GraphStatistics {
    GraphStatistics {
        node_count: nodes.iter().count(),
        edge_count: edges.iter().count(),
        selected_count: selected.iter().filter(|s| s.is_selected).count(),
    }
}

/// Query handler plugin that provides domain query systems
pub struct QueryHandlerPlugin;

impl Plugin for QueryHandlerPlugin {
    fn build(&self, app: &mut App) {
        // Query systems can be registered here if needed
        // Most queries are called directly from other systems
    }
}
