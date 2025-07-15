//! Graph layout algorithms for visualization
//!
//! This module implements various layout algorithms to position nodes in the graph visualization.

use bevy::prelude::*;
use crate::components::{NodeVisual, EdgeVisual};
use crate::resources::{GraphLayoutConfig, ActiveGraph};
use crate::visualization::{LayoutType, VisualizationHints};
use cim_contextgraph::ContextGraphId as GraphId;
use std::collections::HashMap;

/// Resource to track the current layout algorithm for each graph
#[derive(Resource, Default)]
pub struct GraphLayoutState {
    /// Map from graph ID to its current layout algorithm
    pub layout_algorithms: HashMap<GraphId, LayoutType>,
    /// Visualization hints for each graph
    pub visualization_hints: HashMap<GraphId, VisualizationHints>,
}

/// System to apply layout algorithms based on visualization hints
pub fn apply_layout_algorithm(
    mut nodes: Query<(Entity, &NodeVisual, &mut Transform)>,
    edges: Query<&EdgeVisual>,
    layout_config: Res<GraphLayoutConfig>,
    active_graph: Res<ActiveGraph>,
    layout_state: Res<GraphLayoutState>,
    time: Res<Time>,
) {
    if let Some(graph_id) = &active_graph.graph_id {
        // Get the layout algorithm for this graph
        let layout_type = layout_state
            .layout_algorithms
            .get(graph_id)
            .copied()
            .unwrap_or(LayoutType::ForceDirected);
        
        match layout_type {
            LayoutType::ForceDirected => apply_force_directed_layout(
                &mut nodes,
                &edges,
                &layout_config,
                graph_id,
                &time,
            ),
            LayoutType::Hierarchical => apply_hierarchical_layout(
                &mut nodes,
                &edges,
                &layout_config,
                graph_id,
            ),
            LayoutType::Circular => apply_circular_layout(
                &mut nodes,
                &layout_config,
                graph_id,
            ),
            LayoutType::Grid => apply_grid_layout(
                &mut nodes,
                &layout_config,
                graph_id,
            ),
            LayoutType::Random => apply_random_layout(
                &mut nodes,
                graph_id,
            ),
        }
    }
}

/// Apply force-directed layout algorithm
fn apply_force_directed_layout(
    nodes: &mut Query<(Entity, &NodeVisual, &mut Transform)>,
    edges: &Query<&EdgeVisual>,
    config: &GraphLayoutConfig,
    graph_id: &GraphId,
    time: &Time,
) {
    // Collect all nodes for the current graph with their entities
    let mut node_entities: Vec<Entity> = Vec::new();
    let mut node_positions: HashMap<Entity, Vec3> = HashMap::new();
    let mut node_forces: HashMap<Entity, Vec3> = HashMap::new();
    
    // First pass: collect node data
    for (entity, node_visual, transform) in nodes.iter() {
        if &node_visual.graph_id == graph_id {
            node_entities.push(entity);
            node_positions.insert(entity, transform.translation);
            node_forces.insert(entity, Vec3::ZERO);
        }
    }
    
    // Apply repulsive forces between all nodes
    for i in 0..node_entities.len() {
        for j in (i + 1)..node_entities.len() {
            let entity_a = node_entities[i];
            let entity_b = node_entities[j];
            
            let pos_a = node_positions[&entity_a];
            let pos_b = node_positions[&entity_b];
            
            let diff = pos_a - pos_b;
            let distance = diff.length().max(0.1);
            let force_magnitude = config.force_directed_strength / (distance * distance);
            let force = diff.normalize() * force_magnitude;
            
            node_forces.entry(entity_a).and_modify(|f| *f += force);
            node_forces.entry(entity_b).and_modify(|f| *f -= force);
        }
    }
    
    // Apply attractive forces along edges
    for edge_visual in edges.iter() {
        if let (Some(pos_a), Some(pos_b)) = (
            node_positions.get(&edge_visual.source_entity),
            node_positions.get(&edge_visual.target_entity),
        ) {
            let diff = *pos_b - *pos_a;
            let distance = diff.length().max(0.1);
            let force_magnitude = config.force_directed_distance * (distance - 100.0);
            let force = diff.normalize() * force_magnitude;
            
            node_forces.entry(edge_visual.source_entity).and_modify(|f| *f += force);
            node_forces.entry(edge_visual.target_entity).and_modify(|f| *f -= force);
        }
    }
    
    // Apply forces to update positions
    let delta_time = time.delta_secs();
    for (entity, node_visual, mut transform) in nodes.iter_mut() {
        if &node_visual.graph_id == graph_id {
            if let Some(force) = node_forces.get(&entity) {
                transform.translation += *force * delta_time * 0.1;
            }
        }
    }
}

/// Apply hierarchical layout algorithm
fn apply_hierarchical_layout(
    nodes: &mut Query<(Entity, &NodeVisual, &mut Transform)>,
    edges: &Query<&EdgeVisual>,
    config: &GraphLayoutConfig,
    graph_id: &GraphId,
) {
    // Simple hierarchical layout - arrange nodes in layers
    let mut layers: HashMap<Entity, usize> = HashMap::new();
    let mut nodes_by_layer: HashMap<usize, Vec<Entity>> = HashMap::new();
    
    // Collect nodes for this graph
    let mut graph_nodes: Vec<Entity> = Vec::new();
    for (entity, node_visual, _) in nodes.iter() {
        if &node_visual.graph_id == graph_id {
            graph_nodes.push(entity);
            layers.insert(entity, 0);
        }
    }
    
    // Simple layer assignment (could be improved with proper topological sort)
    let mut changed = true;
    while changed {
        changed = false;
        for edge_visual in edges.iter() {
            if let (Some(&source_layer), Some(target_layer)) = (
                layers.get(&edge_visual.source_entity),
                layers.get_mut(&edge_visual.target_entity),
            ) {
                if *target_layer <= source_layer {
                    *target_layer = source_layer + 1;
                    changed = true;
                }
            }
        }
    }
    
    // Group nodes by layer
    for (entity, &layer) in layers.iter() {
        nodes_by_layer.entry(layer).or_insert_with(Vec::new).push(*entity);
    }
    
    // Position nodes by layer
    for (layer, entities) in nodes_by_layer.iter() {
        let count = entities.len() as f32;
        for (i, entity) in entities.iter().enumerate() {
            if let Ok((_, _, mut transform)) = nodes.get_mut(*entity) {
                let x = (i as f32 - count / 2.0) * config.grid_spacing;
                let y = *layer as f32 * config.hierarchical_layer_spacing;
                transform.translation = Vec3::new(x, y, 0.0);
            }
        }
    }
}

/// Apply circular layout algorithm
fn apply_circular_layout(
    nodes: &mut Query<(Entity, &NodeVisual, &mut Transform)>,
    config: &GraphLayoutConfig,
    graph_id: &GraphId,
) {
    let mut node_count = 0;
    let mut node_entities: Vec<Entity> = Vec::new();
    
    // Count nodes for this graph
    for (entity, node_visual, _) in nodes.iter() {
        if &node_visual.graph_id == graph_id {
            node_entities.push(entity);
            node_count += 1;
        }
    }
    
    if node_count == 0 {
        return;
    }
    
    // Position nodes in a circle
    let angle_step = std::f32::consts::TAU / node_count as f32;
    
    for (index, entity) in node_entities.iter().enumerate() {
        if let Ok((_, _, mut transform)) = nodes.get_mut(*entity) {
            let angle = index as f32 * angle_step;
            let x = angle.cos() * config.circular_radius;
            let y = angle.sin() * config.circular_radius;
            transform.translation = Vec3::new(x, y, 0.0);
        }
    }
}

/// Apply grid layout algorithm
fn apply_grid_layout(
    nodes: &mut Query<(Entity, &NodeVisual, &mut Transform)>,
    config: &GraphLayoutConfig,
    graph_id: &GraphId,
) {
    let mut node_count = 0;
    let mut node_entities: Vec<Entity> = Vec::new();
    
    // Count nodes for this graph
    for (entity, node_visual, _) in nodes.iter() {
        if &node_visual.graph_id == graph_id {
            node_entities.push(entity);
            node_count += 1;
        }
    }
    
    if node_count == 0 {
        return;
    }
    
    // Calculate grid dimensions
    let grid_size = (node_count as f32).sqrt().ceil() as usize;
    
    // Position nodes in a grid
    for (index, entity) in node_entities.iter().enumerate() {
        if let Ok((_, _, mut transform)) = nodes.get_mut(*entity) {
            let row = index / grid_size;
            let col = index % grid_size;
            
            let x = (col as f32 - grid_size as f32 / 2.0) * config.grid_spacing;
            let y = (row as f32 - grid_size as f32 / 2.0) * config.grid_spacing;
            
            transform.translation = Vec3::new(x, y, 0.0);
        }
    }
}

/// Apply random layout algorithm
fn apply_random_layout(
    nodes: &mut Query<(Entity, &NodeVisual, &mut Transform)>,
    graph_id: &GraphId,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for (_, node_visual, mut transform) in nodes.iter_mut() {
        if &node_visual.graph_id == graph_id {
            let x = rng.gen_range(-500.0..500.0);
            let y = rng.gen_range(-500.0..500.0);
            let z = rng.gen_range(-100.0..100.0);
            transform.translation = Vec3::new(x, y, z);
        }
    }
}

/// System to update layout algorithm from visualization hints
pub fn update_layout_from_hints(
    mut layout_state: ResMut<GraphLayoutState>,
    active_graph: Res<ActiveGraph>,
) {
    if let Some(graph_id) = &active_graph.graph_id {
        // Check if we have visualization hints for this graph
        if let Some(hints) = layout_state.visualization_hints.get(graph_id) {
            // Update the layout algorithm
            layout_state.layout_algorithms.insert(*graph_id, hints.layout_algorithm);
        }
    }
}

/// Command to set layout algorithm for a graph
#[derive(Event)]
pub struct SetLayoutAlgorithm {
    pub graph_id: GraphId,
    pub layout_type: LayoutType,
}

/// System to handle layout algorithm change commands
pub fn handle_layout_commands(
    mut layout_state: ResMut<GraphLayoutState>,
    mut events: EventReader<SetLayoutAlgorithm>,
) {
    for event in events.read() {
        layout_state.layout_algorithms.insert(event.graph_id, event.layout_type);
        info!("Changed layout algorithm for graph {:?} to {:?}", event.graph_id, event.layout_type);
    }
}