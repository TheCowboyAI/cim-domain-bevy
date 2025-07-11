//! Systems for managing edge states and visualization
//!
//! This module provides systems for updating and managing edge states based on various conditions.

use bevy::prelude::*;
use crate::components::{EdgeVisual, EdgeState, EdgeStyle, FlowDirection};
use crate::resources::ActiveGraph;

/// System to update edge visualization based on edge state
pub fn update_edge_visualization(
    mut edges: Query<(&EdgeVisual, &EdgeState, &mut EdgeStyle), Changed<EdgeState>>,
    active_graph: Res<ActiveGraph>,
) {
    if let Some(graph_id) = &active_graph.graph_id {
        for (edge_visual, edge_state, mut edge_style) in edges.iter_mut() {
            if &edge_visual.graph_id == graph_id {
                // Update color based on state
                if edge_state.is_highlighted {
                    edge_style.color = Color::srgb(1.0, 0.8, 0.0); // Highlight color
                    edge_style.thickness = 0.15;
                } else if edge_state.is_active {
                    edge_style.color = Color::srgb(0.5, 0.5, 0.5); // Normal color
                    edge_style.thickness = 0.1;
                } else {
                    edge_style.color = Color::srgba(0.3, 0.3, 0.3, 0.5); // Inactive color
                    edge_style.thickness = 0.05;
                }
                
                // Update thickness based on weight
                edge_style.thickness *= edge_state.weight;
                
                // Update arrow size based on flow direction
                match edge_state.flow_direction {
                    FlowDirection::Forward => edge_style.arrow_size = 0.2,
                    FlowDirection::Backward => edge_style.arrow_size = 0.2,
                    FlowDirection::Bidirectional => edge_style.arrow_size = 0.3,
                }
            }
        }
    }
}

/// System to highlight edges connected to selected nodes
pub fn highlight_connected_edges(
    nodes: Query<(Entity, &crate::components::NodeVisual), With<crate::components::Selected>>,
    mut edges: Query<(&EdgeVisual, &mut EdgeState)>,
) {
    // First, reset all edge highlights
    for (_, mut edge_state) in edges.iter_mut() {
        edge_state.is_highlighted = false;
    }
    
    // Then highlight edges connected to selected nodes
    let selected_entities: Vec<Entity> = nodes.iter().map(|(e, _)| e).collect();
    
    for (edge_visual, mut edge_state) in edges.iter_mut() {
        if selected_entities.contains(&edge_visual.source_entity) || 
           selected_entities.contains(&edge_visual.target_entity) {
            edge_state.is_highlighted = true;
        }
    }
}

/// System to update edge weights based on usage or other metrics
pub fn update_edge_weights(
    mut edges: Query<(&EdgeVisual, &mut EdgeState)>,
    time: Res<Time>,
) {
    // Simple example: decay edge weights over time
    let decay_rate = 0.99_f32.powf(time.delta_secs());
    
    for (_, mut edge_state) in edges.iter_mut() {
        edge_state.weight *= decay_rate;
        edge_state.weight = edge_state.weight.max(0.1); // Minimum weight
    }
}

/// Event for edge state changes
#[derive(Event)]
pub struct EdgeStateChanged {
    pub edge_id: cim_contextgraph::EdgeId,
    pub new_state: EdgeState,
}

/// System to handle edge state change events
pub fn handle_edge_state_changes(
    mut events: EventReader<EdgeStateChanged>,
    mut edges: Query<(&EdgeVisual, &mut EdgeState)>,
) {
    for event in events.read() {
        for (edge_visual, mut edge_state) in edges.iter_mut() {
            if edge_visual.edge_id == event.edge_id {
                *edge_state = event.new_state.clone();
                break;
            }
        }
    }
}

/// System to animate edge flow visualization
pub fn animate_edge_flow(
    mut edges: Query<(&EdgeVisual, &EdgeState, &mut EdgeStyle)>,
    time: Res<Time>,
) {
    let phase = (time.elapsed_secs() * 2.0).sin() * 0.5 + 0.5;
    
    for (_, edge_state, mut edge_style) in edges.iter_mut() {
        if edge_state.is_active {
            // Animate dashed pattern or color intensity based on flow
            let intensity = match edge_state.flow_direction {
                FlowDirection::Forward => phase,
                FlowDirection::Backward => 1.0 - phase,
                FlowDirection::Bidirectional => (phase * 2.0).sin().abs(),
            };
            
            // Modulate the edge color alpha for flow visualization
            edge_style.color.set_alpha(0.5 + intensity * 0.5);
        }
    }
}